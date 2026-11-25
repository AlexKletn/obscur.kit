#![doc = include_str!("../docs/api/smoke_wasm.doc/_module.md")]

use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use obscur_plugin_executor_wasm::executor::{WasmLimits, WasmMessageLimits, WasmPluginInstance};
use obscur_plugin_host::{
    capability::ServiceScope,
    service::{
        JsonServiceBridge, ServiceFrameCodec, ServicePacketDispatch, ServiceRegistry,
        ServiceSession,
    },
};
use obscur_plugin_model::{identity::PluginPrincipal, manifest::Version};
use obscur_plugin_package::package::PluginPackage;
use serde_json::json;

#[derive(Default)]
pub(crate) struct Requests {
    pub(crate) initial: Option<PathBuf>,
    pub(crate) dispatch: Option<PathBuf>,
    pub(crate) mock_response: Option<PathBuf>,
}

pub(crate) fn run(
    package: &PluginPackage,
    requests: &Requests,
    mut output: impl Write,
) -> Result<(), Box<dyn Error>> {
    let limits = WasmLimits {
        module_bytes: 16 * 1024 * 1024,
        memory_bytes: 64 * 1024 * 1024,
        table_elements: 4096,
        fuel_per_call: 1_000_000,
    };
    let messages = WasmMessageLimits {
        request_bytes: 64 * 1024,
        response_bytes: 64 * 1024,
    };
    let initial = requests
        .initial
        .as_deref()
        .map(|path| read_request(path, messages.request_bytes))
        .transpose()?
        .unwrap_or_default();
    let dispatch = requests
        .dispatch
        .as_deref()
        .map(|path| read_request(path, messages.request_bytes))
        .transpose()?;
    let mock = requests
        .mock_response
        .as_deref()
        .map(|path| read_request(path, messages.response_bytes))
        .transpose()?;
    let (instance, initialized, dispatched) = if let Some(mock) = mock {
        run_with_mock(
            package,
            limits,
            messages,
            initial,
            dispatch.as_deref(),
            serde_json::from_slice(&mock)?,
        )?
    } else {
        let (mut instance, initialized) =
            WasmPluginInstance::activate(package, limits, messages, &initial)?;
        let dispatched = dispatch
            .as_deref()
            .map(|message| instance.exchange(message))
            .transpose()?;
        (instance, initialized, dispatched)
    };
    let deactivated = instance.deactivate(&[])?;
    serde_json::to_writer_pretty(
        &mut output,
        &json!({
            "id": package.manifest().id.as_str(),
            "version": package.manifest().version,
            "initialize_reply": initialized,
            "dispatch_reply": dispatched,
            "deactivate_reply": deactivated,
        }),
    )?;
    writeln!(output)?;
    Ok(())
}

type SmokeReplies = (WasmPluginInstance, Vec<u8>, Option<Vec<u8>>);

fn run_with_mock(
    package: &PluginPackage,
    limits: WasmLimits,
    messages: WasmMessageLimits,
    initial: Vec<u8>,
    dispatch: Option<&[u8]>,
    response: serde_json::Value,
) -> Result<SmokeReplies, Box<dyn Error>> {
    let principal = PluginPrincipal::new(package.manifest().id.clone(), "cli-smoke");
    let scope = ServiceScope::new("cli.mock", Version::new(1, 0, 0), ["get"]);
    let mut registry = ServiceRegistry::<serde_json::Value, serde_json::Value>::default();
    registry.register(principal.clone(), scope.clone(), move |_| {
        std::future::ready(Ok(response.clone()))
    })?;
    let mut session = ServiceSession::new(principal, 1);
    let handle = session.issue(&registry, scope, None)?;
    let request = [handle.to_be_bytes().as_slice(), &initial].concat();
    let (mut instance, initialized) = WasmPluginInstance::activate_with_host_calls(
        package,
        limits,
        messages,
        &request,
        messages.request_bytes,
    )?;
    let bridge = JsonServiceBridge::new(&registry, messages.request_bytes, messages.response_bytes);
    let mut calls_left = 16;
    let initialized = pump_mock(
        &mut instance,
        &session,
        &bridge,
        initialized,
        &mut calls_left,
    )?;
    let dispatched = if let Some(message) = dispatch {
        let reply = instance.exchange(message)?;
        Some(pump_mock(
            &mut instance,
            &session,
            &bridge,
            reply,
            &mut calls_left,
        )?)
    } else {
        None
    };
    Ok((instance, initialized, dispatched))
}

fn pump_mock(
    instance: &mut WasmPluginInstance,
    session: &ServiceSession,
    bridge: &impl ServicePacketDispatch,
    mut reply: Vec<u8>,
    calls_left: &mut usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };

    let codec = ServiceFrameCodec::new(64 * 1024 - 4);
    while let Some(packet) = instance.take_host_call() {
        *calls_left = calls_left
            .checked_sub(1)
            .ok_or("mock host call budget exceeded")?;
        let mut call = std::pin::pin!(bridge.dispatch_packet(codec, session, &packet));
        let Poll::Ready(result) = call.as_mut().poll(&mut Context::from_waker(Waker::noop()))
        else {
            return Err("mock service did not complete synchronously".into());
        };
        reply = instance.exchange(&result?)?;
    }
    Ok(reply)
}

fn read_request(path: &Path, limit: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(format!("request file exceeds {limit} bytes: {}", path.display()).into());
    }
    Ok(bytes)
}
