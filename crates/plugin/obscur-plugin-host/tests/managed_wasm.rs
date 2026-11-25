//! WASM lifecycle through the transactional plugin manager.

#![cfg(feature = "plugin-wasm")]

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_executor_wasm::executor::{ManagedWasmPlugin, WasmLimits, WasmMessageLimits};
use obscur_plugin_host::manager::PluginManager;
use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::{ContributionId, PluginId},
    lifecycle::PluginLifecycleState,
    manifest::Version,
};
use obscur_plugin_package::{
    package::{PackageEntry, PackageReader},
    path::PackagePath,
    validation::{PackageError, PackageLimits, PackageValidator},
};
use obscur_routing::{route::Route, router::RouteRegistry};
use obscur_view::view::{View, ViewNode};
use serde_json::json;
use sha2::{Digest, Sha256};

fn point() -> ExtensionPoint {
    ExtensionPoint::new("app.views").unwrap()
}

fn publish_point(host: &mut PluginManager) {
    assert!(host.publish_with(point(), |plugin, view| {
        Ok(Some(Route::new(format!(
            "/integrations/{}/{}",
            plugin.as_str(),
            view.id().as_str()
        ))))
    }));
}

struct Entries(Vec<PackageEntry>);

impl PackageReader for Entries {
    fn read_entries(self, _: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
        Ok(self.0)
    }
}

fn plugin(text: &'static str, reject: bool) -> ManagedWasmPlugin {
    let bytes = wat::parse_str(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
        (global $ready (mut i32) (i32.const 0))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32)
            i32.const 1 global.set $ready local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32)
            global.get $ready i32.eqz if unreachable end
            i32.const 0 global.set $ready i32.const 0))"#,
    )
    .unwrap();
    plugin_from_bytes(bytes, text, reject)
}

fn plugin_from_bytes(bytes: Vec<u8>, text: &'static str, reject: bool) -> ManagedWasmPlugin {
    plugin_with_fuel(bytes, text, reject, 1_000)
}

fn plugin_with_fuel(
    bytes: Vec<u8>,
    text: &'static str,
    reject: bool,
    fuel: u64,
) -> ManagedWasmPlugin {
    let path = "main.wasm";
    let manifest = json!({
        "schema_version": 1, "id": "example.wasm", "name": "Example", "author": "Example",
        "version": "1.0.0", "kind": "wasm", "compatibility": { "plugin_api": "^1", "ui_ir": "^1" },
        "entrypoint": { "path": path },
        "files": [{ "path": path, "sha256": format!("{:x}", Sha256::digest(&bytes)) }]
    });
    let package = PackageValidator::new(Version::new(1, 0, 0), Version::new(1, 0, 0))
        .load(Entries(vec![
            PackageEntry::file(PackagePath::parse(path).unwrap(), bytes),
            PackageEntry::file(
                PackagePath::parse("manifest.json").unwrap(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
        ]))
        .unwrap();
    ManagedWasmPlugin::new(
        package,
        WasmLimits {
            module_bytes: 4096,
            memory_bytes: 65536,
            table_elements: 0,
            fuel_per_call: fuel,
        },
        WasmMessageLimits {
            request_bytes: 64,
            response_bytes: 16,
        },
        text.as_bytes().to_vec(),
        move |message, registrations| {
            assert_eq!(message, text.as_bytes());
            registrations.contribute_view(
                point(),
                ViewContribution::new(ContributionId::new("Page").unwrap(), text),
                move || View::single(ViewNode::text(text)),
            );
            if reject {
                return Err(PluginApiError::Rejected("invalid guest UI".into()));
            }
            Ok(())
        },
    )
}

#[test]
fn reports_wasm_traps_and_execution_quota_as_distinct_activation_errors() {
    let mut manager = PluginManager::new(RouteRegistry::new());
    let id = PluginId::new("example.wasm").unwrap();
    let mut activate = |initializer: &str| {
        let bytes = wat::parse_str(format!(
            r#"(module
            (memory (export "memory") 1)
            (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
            (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
            (func (export "obscur_plugin_initialize_v1") (param i32) (result i32)
                {initializer})
            (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
            (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0))"#
        ))
        .unwrap();
        manager.activate_wasm(plugin_from_bytes(bytes, "", false))
    };
    assert!(matches!(
        activate("unreachable"),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
    assert!(matches!(
        activate("(loop br 0) i32.const 0"),
        Err(PluginApiError::ExecutionQuotaExceeded(_))
    ));
    assert_eq!(manager.state(&id), None);
    assert!(manager.contributions().views().is_empty());
}

#[test]
fn runtime_fault_withdraws_the_wasm_view_and_blocks_restart() {
    use obscur_plugin_host::lifecycle::CrashRecovery;
    use std::time::Duration;

    let bytes = wat::parse_str(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) unreachable)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0))"#,
    )
    .unwrap();
    let guest = plugin_from_bytes(bytes, "old", false);
    let id = guest.id().clone();
    let selected = guest.manifest().clone();
    let handle = guest.handle();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    manager.set_crash_recovery(
        id.clone(),
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 1),
    );
    let mut wrong = selected.clone();
    wrong.version = Version::new(2, 0, 0);
    let view = ContributionId::new("Page").unwrap();
    assert!(matches!(
        manager.open_wasm_manifest_view(&wrong, &point(), &view, |_| Ok(plugin("other", false))),
        Err(PluginApiError::Rejected(_))
    ));
    assert!(
        manager
            .open_wasm_manifest_view(&selected, &point(), &view, |_| Ok(guest))
            .unwrap()
            .is_some()
    );
    assert!(handle.exchange(&[]).is_err());
    assert!(handle.is_closed());
    let faults = manager.withdraw_faulted_wasm();
    assert!(
        matches!(faults.as_slice(), [(owner, PluginApiError::ExecutorCrashed(_))] if *owner == id)
    );
    let failure = manager.last_failure(&id).unwrap().clone();
    assert_eq!(failure.version, Some(Version::new(1, 0, 0)));
    assert_eq!(failure.error, faults[0].1);
    assert!(manager.contributions().views().is_empty());
    assert!(manager.withdraw_faulted_wasm().is_empty());
    assert_eq!(
        manager.activate_wasm(plugin("new", false)),
        Err(PluginApiError::RestartBlocked)
    );

    #[cfg(any(unix, windows))]
    {
        use obscur_plugin_host::storage::{
            FilesystemStateStore, PluginStorage, StateStore, StorageNamespace, StorageQuota,
        };
        use obscur_plugin_model::{manifest::VersionReq, state::StateSchema};

        let directory = tempfile::tempdir().unwrap();
        let store = FilesystemStateStore::open(directory.path(), StorageQuota::new(1024)).unwrap();
        let namespace = StorageNamespace::new(id.clone(), "example-installation");
        let snapshot = PluginStorage::new(
            namespace.clone(),
            StateSchema::new(Version::new(1, 0, 0)),
            b"committed".to_vec(),
        );
        assert!(store.compare_exchange(None, &snapshot).unwrap());
        manager.crash_recovery_mut(&id).unwrap().reset();
        let fresh = || plugin("committed", false).with_initial_request(b"fallback".to_vec());
        assert!(matches!(
            manager.activate_wasm_from_state(
                fresh(),
                &store,
                &namespace,
                &VersionReq::parse("^2").unwrap()
            ),
            Err(PluginApiError::Rejected(_))
        ));
        manager
            .activate_wasm_from_state(
                fresh(),
                &store,
                &namespace,
                &VersionReq::parse("^1").unwrap(),
            )
            .unwrap();
        assert_eq!(manager.last_failure(&id), Some(&failure));
        assert_eq!(
            manager.contributions().views()[0].definition().title(),
            "committed"
        );
    }
}

#[test]
fn confirming_a_faulted_wasm_replacement_restores_the_previous_instance() {
    let bytes = wat::parse_str(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) unreachable)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0))"#,
    )
    .unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    let previous = plugin("old", false);
    let previous_handle = previous.handle();
    manager.activate_wasm(previous).unwrap();

    let replacement = plugin_from_bytes(bytes, "new", false);
    let handle = replacement.handle();
    let handover = manager.stage_wasm_handover(replacement).unwrap();
    assert!(handle.exchange(&[]).is_err());
    assert!(matches!(
        handover.commit(),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
    assert_eq!(
        manager.contributions().views()[0].definition().title(),
        "old"
    );
    assert!(previous_handle.exchange(b"still here").is_ok());
    assert!(manager.withdraw_faulted_wasm().is_empty());
}

#[test]
fn shutdown_request_reaches_the_optional_wasm_export() {
    use std::time::{Duration, Instant};

    let bytes = wat::parse_str(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0)
        (func (export "obscur_plugin_shutdown_requested_v1") (param i32) (result i32)
            local.get 0 i32.const 8 i32.ne if unreachable end
            i32.const 16 i64.load i64.eqz if unreachable end
            i32.const 16 i64.const -1 i64.store
            i32.const 24 i32.const 65 i32.store8
            i32.const 9))"#,
    )
    .unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    manager
        .activate_wasm(plugin_from_bytes(bytes, "", false))
        .unwrap();
    let budget = Duration::from_secs(5);
    assert!(matches!(
        manager.shutdown_requested(Instant::now() + budget).as_slice(),
        [(_, Ok(Some(concern)))]
            if concern.message.as_ref() == "A"
                && !concern.requested_time.is_zero()
                && concern.requested_time <= budget
    ));
    assert!(manager.deactivate_all().is_empty());
}

#[test]
fn shutdown_interrupts_wasm_with_fuel_left_at_the_deadline() {
    use std::time::{Duration, Instant};

    let bytes = wat::parse_str(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32)
            (loop br 0) i32.const 0)
        (func (export "obscur_plugin_shutdown_requested_v1") (param i32) (result i32)
            (loop br 0) i32.const 0))"#,
    )
    .unwrap();
    let guest = plugin_with_fuel(bytes.clone(), "", false, 1_000_000_000);
    let handle = guest.handle();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    manager.activate_wasm(guest).unwrap();

    assert!(matches!(
        manager
            .shutdown_requested(Instant::now() + Duration::from_millis(20))
            .as_slice(),
        [(_, Err(PluginApiError::ExecutionTimedOut))]
    ));
    assert!(handle.is_faulted());
    assert!(matches!(
        manager.withdraw_faulted_wasm().as_slice(),
        [(_, PluginApiError::ExecutionTimedOut)]
    ));

    let guest = plugin_with_fuel(bytes, "", false, 1_000_000_000);
    let handle = guest.handle();
    manager.activate_wasm(guest).unwrap();
    assert!(matches!(
        manager
            .deactivate_all_until(Instant::now() + Duration::from_millis(20))
            .as_slice(),
        [(_, PluginApiError::ExecutionTimedOut)]
    ));
    assert!(handle.is_closed());
    assert!(manager.contributions().views().is_empty());

    manager.activate_wasm(plugin("old", false)).unwrap();
    assert!(matches!(
        manager.deactivate_all_until(Instant::now()).as_slice(),
        [(_, PluginApiError::ExecutionTimedOut)]
    ));
    assert!(manager.contributions().views().is_empty());
}

#[test]
fn activates_and_updates_wasm_pages_through_the_manager() {
    let id = PluginId::new("example.wasm").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    assert_eq!(
        manager.activate_wasm(plugin("rejected", true)),
        Err(PluginApiError::Rejected("invalid guest UI".into()))
    );
    assert!(manager.contributions().views().is_empty());
    assert!(manager.state(&id).is_none());
    let original = plugin("old", false);
    let handle = original.handle();
    manager.activate_wasm(original).unwrap();
    manager.suspend(&id).unwrap();
    assert_eq!(manager.state(&id), Some(PluginLifecycleState::Suspended));
    assert!(manager.contributions().views().is_empty());
    assert!(
        !manager
            .navigate("/integrations/example.wasm/Page")
            .is_matched()
    );
    assert!(handle.exchange(b"paused").is_err());
    assert_eq!(
        manager.activate_wasm(plugin("other", false)),
        Err(PluginApiError::AlreadyActive)
    );
    manager.resume(&id).unwrap();
    assert_eq!(handle.exchange(b"resumed").unwrap(), b"resumed");
    assert_eq!(manager.state(&id), Some(PluginLifecycleState::Active));
    assert_eq!(
        manager.plugin_states().next(),
        Some((
            &id,
            Some(&Version::new(1, 0, 0)),
            PluginLifecycleState::Active
        ))
    );
    assert!(
        manager
            .navigate("/integrations/example.wasm/Page")
            .is_matched()
    );
    assert!(manager.current_view().unwrap().is_some());
    let handover = manager.stage_wasm_handover(plugin("new", false)).unwrap();
    assert_eq!(
        handover.trial().contributions().views()[0]
            .definition()
            .title(),
        "new"
    );
    handover.rollback().unwrap();
    assert_eq!(
        manager.contributions().views()[0].definition().title(),
        "old"
    );

    manager
        .stage_wasm_handover(plugin("new", false))
        .unwrap()
        .commit()
        .unwrap();
    assert_eq!(
        manager.contributions().views()[0].definition().title(),
        "new"
    );
    manager.deactivate(&id).unwrap();
    assert!(manager.current_view().unwrap().is_none());
}

#[test]
fn managed_message_handles_follow_activation_rollback_and_commit() {
    use obscur_plugin_executor_wasm::error::WasmExecutorError;

    let id = PluginId::new("example.wasm").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);

    let rejected = plugin("rejected", true);
    let failed = rejected.handle();
    assert!(manager.activate_wasm(rejected).is_err());
    assert_eq!(failed.exchange(b"test"), Err(WasmExecutorError::Inactive));

    let original = plugin("old", false);
    let old = original.handle();
    assert_eq!(old.exchange(b"before"), Err(WasmExecutorError::Inactive));
    manager.activate_wasm(original).unwrap();
    assert_eq!(old.exchange(b"active").unwrap(), b"active");

    let replacement = plugin("new", false);
    let trial = replacement.handle();
    let handover = manager.stage_wasm_handover(replacement).unwrap();
    assert_eq!(trial.exchange(b"trial").unwrap(), b"trial");
    handover.rollback().unwrap();
    assert_eq!(trial.exchange(b"gone"), Err(WasmExecutorError::Inactive));
    assert_eq!(old.exchange(b"old").unwrap(), b"old");

    let replacement = plugin("new", false);
    let active = replacement.handle();
    manager
        .stage_wasm_handover(replacement)
        .unwrap()
        .commit()
        .unwrap();
    assert_eq!(old.exchange(b"gone"), Err(WasmExecutorError::Inactive));
    assert_eq!(active.exchange(b"new").unwrap(), b"new");
    manager.deactivate(&id).unwrap();
    assert_eq!(active.exchange(b"gone"), Err(WasmExecutorError::Inactive));
}

#[test]
fn guest_initiated_calls_use_the_hosts_session_and_deliver_a_reply() {
    use obscur_plugin_host::{
        capability::ServiceScope,
        service::{
            JsonServiceBridge, JsonServiceRouter, ServiceCallFrame, ServiceFrameCodec,
            ServiceRegistry, ServiceSession, ServiceWireError, WasmServiceDriver,
            WasmServiceDriverError,
        },
    };
    use obscur_plugin_model::identity::PluginPrincipal;
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };

    let bytes = wat::parse_str(
        r#"(module
        (import "obscur_host" "submit_v1" (func $submit (param i32 i32) (result i32)))
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 64))
        (global $waiting (mut i32) (i32.const 0))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32)
            global.get $waiting
            if (result i32)
                i32.const 0 global.set $waiting
                local.get 0
            else
                i32.const 16 local.get 0 call $submit
                if unreachable end
                i32.const 1 global.set $waiting
                i32.const 0
            end)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0))"#,
    )
    .unwrap();
    let plugin = plugin_from_bytes(bytes.clone(), "guest", false).with_host_calls(64);
    let handle = plugin.handle();
    let principal = PluginPrincipal::new(plugin.id().clone(), "verified-installation");
    let mut manager = PluginManager::new(RouteRegistry::new());
    publish_point(&mut manager);
    manager.activate_wasm(plugin).unwrap();

    let scope = || ServiceScope::new("numbers", Version::new(1, 0, 0), ["get"]);
    let mut services = ServiceRegistry::<u32, u32>::default();
    let release = std::rc::Rc::new(std::cell::Cell::new(false));
    let gate = release.clone();
    services
        .register(principal.clone(), scope(), move |request| {
            let gate = gate.clone();
            async move {
                assert_eq!(request.caller.installation(), "verified-installation");
                if request.payload == 0 {
                    std::future::pending::<()>().await;
                }
                if request.payload == 1 {
                    std::future::poll_fn(|_| {
                        if gate.get() {
                            Poll::Ready(())
                        } else {
                            Poll::Pending
                        }
                    })
                    .await;
                }
                Ok(request.payload + 1)
            }
        })
        .unwrap();
    let status_scope = || ServiceScope::new("status", Version::new(1, 0, 0), ["get"]);
    let mut status = ServiceRegistry::<bool, String>::default();
    status
        .register(principal.clone(), status_scope(), |request| async move {
            Ok(if request.payload { "ready" } else { "offline" }.to_owned())
        })
        .unwrap();
    let mut session = ServiceSession::new(principal.clone(), 2);
    let status_handle = session.issue(&status, status_scope(), None).unwrap();
    let mut foreign = ServiceSession::new(
        PluginPrincipal::new(principal.plugin().clone(), "foreign"),
        1,
    );
    let allowed = session.issue(&services, scope(), None).unwrap();
    let disallowed = foreign.issue(&services, scope(), None).unwrap();
    let codec = ServiceFrameCodec::new(60);
    let wrong = ServiceSession::new(
        PluginPrincipal::new(PluginId::new("other.plugin").unwrap(), "foreign"),
        1,
    );
    assert!(matches!(
        WasmServiceDriver::new(handle.clone(), wrong, codec),
        Err(WasmServiceDriverError::PrincipalMismatch)
    ));
    let mut driver = WasmServiceDriver::new(handle, session, codec).unwrap();
    let mut router = JsonServiceRouter::default();
    router
        .mount(
            "numbers",
            Version::new(1, 0, 0),
            JsonServiceBridge::new(&services, 48, 48),
        )
        .unwrap();
    router
        .mount(
            "status",
            Version::new(1, 0, 0),
            JsonServiceBridge::new(&status, 48, 48),
        )
        .unwrap();
    for (id, payload, expected) in [
        (allowed, b"41".as_slice(), Ok(b"42".as_slice())),
        (
            status_handle,
            b"true".as_slice(),
            Ok(br#""ready""#.as_slice()),
        ),
        (disallowed, b"41".as_slice(), Err(ServiceWireError::Denied)),
    ] {
        let mut call = Vec::new();
        codec
            .write_call(
                &mut call,
                ServiceCallFrame {
                    handle: id,
                    operation: "get",
                    payload,
                },
            )
            .unwrap();
        assert!(driver.exchange(&call).unwrap().is_empty());
        assert!(matches!(
            driver.exchange(&[]),
            Err(WasmServiceDriverError::Guest(
                obscur_plugin_executor_wasm::error::WasmExecutorError::HostCallPending
            ))
        ));
        let delivered = match std::pin::pin!(driver.pump_next(&router))
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(result) => result.unwrap().unwrap(),
            Poll::Pending => panic!("test service should complete immediately"),
        };
        assert_eq!(
            ServiceFrameCodec::decode_reply(codec.decode_packet(&delivered).unwrap()).unwrap(),
            expected
        );
    }
    let mut hanging = Vec::new();
    let mut paused = Vec::new();
    codec
        .write_call(
            &mut paused,
            ServiceCallFrame {
                handle: allowed,
                operation: "get",
                payload: b"1",
            },
        )
        .unwrap();
    driver.exchange(&paused).unwrap();
    {
        let mut call = std::pin::pin!(driver.pump_next(&router));
        let mut context = Context::from_waker(Waker::noop());
        assert!(call.as_mut().poll(&mut context).is_pending());
        manager.suspend(principal.plugin()).unwrap();
        release.set(true);
        assert!(matches!(
            call.as_mut().poll(&mut context),
            Poll::Ready(Ok(None))
        ));
    }
    assert!(!driver.revoker().is_revoked());
    manager.resume(principal.plugin()).unwrap();
    assert!(matches!(
        driver.exchange(&[]),
        Err(WasmServiceDriverError::Guest(
            obscur_plugin_executor_wasm::error::WasmExecutorError::HostCallPending
        ))
    ));
    let delivered = match std::pin::pin!(driver.pump_next(&router))
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(Ok(Some(reply))) => reply,
        other => panic!("expected deferred reply, got {other:?}"),
    };
    assert_eq!(
        ServiceFrameCodec::decode_reply(codec.decode_packet(&delivered).unwrap()).unwrap(),
        Ok(b"2".as_slice())
    );
    codec
        .write_call(
            &mut hanging,
            ServiceCallFrame {
                handle: allowed,
                operation: "get",
                payload: b"0",
            },
        )
        .unwrap();
    driver.exchange(&hanging).unwrap();
    let revoke = driver.revoker();
    {
        let mut call = std::pin::pin!(driver.pump_next(&router));
        assert!(
            call.as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
    }
    assert!(revoke.is_revoked());
    assert!(matches!(
        driver.exchange(b"retry"),
        Err(WasmServiceDriverError::SessionRevoked)
    ));
    manager.deactivate(principal.plugin()).unwrap();

    let plugin = plugin_from_bytes(bytes, "guest", false).with_host_calls(64);
    let handle = plugin.handle();
    manager.activate_wasm(plugin).unwrap();
    let mut session = ServiceSession::new(principal.clone(), 1);
    let allowed = session.issue(&services, scope(), None).unwrap();
    let mut driver = WasmServiceDriver::new(handle.clone(), session, codec).unwrap();
    let revoke = driver.revoker();
    let mut request = Vec::new();
    codec
        .write_call(
            &mut request,
            ServiceCallFrame {
                handle: allowed,
                operation: "get",
                payload: b"0",
            },
        )
        .unwrap();
    driver.exchange(&request).unwrap();
    let mut call = std::pin::pin!(driver.pump_next(&router));
    let mut context = Context::from_waker(Waker::noop());
    assert!(call.as_mut().poll(&mut context).is_pending());
    manager.deactivate(principal.plugin()).unwrap();
    assert!(handle.is_closed());
    assert!(matches!(
        call.as_mut().poll(&mut context),
        Poll::Ready(Err(WasmServiceDriverError::Guest(
            obscur_plugin_executor_wasm::error::WasmExecutorError::Inactive
        )))
    ));
    assert!(revoke.is_revoked());
}
