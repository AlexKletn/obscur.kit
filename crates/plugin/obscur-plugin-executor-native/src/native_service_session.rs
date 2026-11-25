//! Scoped service dispatch for a trusted native guest.

use std::{
    future::Future,
    io,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use obscur_plugin_abi::{
    boundary::PanicBoundary,
    function_table::HostFunctionTable,
    value::{AbiBuffer, AbiCallResult, AbiHandle, AbiSlice, AbiStatus},
};
use obscur_plugin_host::service::{
    JsonServiceBridge, ServiceFrameCodec, ServiceRegistry, ServiceSession,
};
use obscur_plugin_model::identity::PluginPrincipal;
use serde::{Serialize, de::DeserializeOwned};

const MAX_PACKET: usize = HostFunctionTable::SERVICE_PACKET_BYTES;
type Dispatch = dyn Fn(&[u8]) -> io::Result<Vec<u8>>;

mod call_scope;
use call_scope::CallScope;

/// Binds an authenticated service session to the current native initialization call on its host thread.
pub struct NativeServiceSession {
    principal: PluginPrincipal,
    dispatch: Rc<Dispatch>,
}

impl NativeServiceSession {
    /// Uses the normal version-1 service packet protocol and checks existing session grants. Handlers must return immediately when polled. The guest supplies a reply buffer of at least 64 KiB.
    pub fn new<Req: DeserializeOwned + 'static, Res: Serialize + 'static>(
        session: ServiceSession,
        registry: Rc<ServiceRegistry<Req, Res>>,
    ) -> Self {
        let principal = session.principal().clone();
        let dispatch = Rc::new(move |packet: &[u8]| {
            let codec = ServiceFrameCodec::new((MAX_PACKET - 4) as u32);
            // The reply has a four-byte length prefix and two-byte protocol header.
            let bridge = JsonServiceBridge::new(&registry, MAX_PACKET, MAX_PACKET - 6);
            let mut call = std::pin::pin!(bridge.dispatch_packet(codec, &session, packet));
            match call.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                Poll::Ready(reply) => reply,
                Poll::Pending => Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native service must complete synchronously",
                )),
            }
        });
        Self {
            principal,
            dispatch,
        }
    }

    pub(super) const fn principal(&self) -> &PluginPrincipal {
        &self.principal
    }

    pub(super) const fn table() -> HostFunctionTable {
        HostFunctionTable::new(call_host)
    }

    pub(super) fn enter<T>(&self, handle: AbiHandle, callback: impl FnOnce() -> T) -> T {
        let _scope = CallScope::enter(handle, Rc::clone(&self.dispatch));
        callback()
    }
}

unsafe extern "C" fn call_host(
    handle: AbiHandle,
    request: AbiSlice,
    output: AbiBuffer,
) -> AbiCallResult {
    let mut written = 0;
    let status = PanicBoundary::status(|| {
        let Some(dispatch) = CallScope::dispatch_for(handle) else {
            return AbiStatus::REJECTED;
        };
        // SAFETY: the guest keeps the input initialized and readable for this call.
        let Some(request) = (unsafe { request.as_bytes(MAX_PACKET) }) else {
            return AbiStatus::REJECTED;
        };
        // Require space for the largest possible reply before invoking a service.
        if output.data.is_null()
            || output.capacity < MAX_PACKET
            || output.capacity > isize::MAX as usize
        {
            return AbiStatus::REJECTED;
        }
        let Ok(reply) = dispatch(request) else {
            return AbiStatus::REJECTED;
        };
        // SAFETY: the guest owns writable output for the call, disjoint from the host's reply.
        if !unsafe { output.write(&reply) } {
            return AbiStatus::REJECTED;
        }
        written = reply.len();
        AbiStatus::OK
    });
    AbiCallResult { status, written }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_plugin_host::{capability::ServiceScope, service::ServiceCallFrame};
    use obscur_plugin_model::{identity::PluginId, manifest::Version};

    #[test]
    fn routes_granted_calls_only_inside_the_current_instance() {
        let principal = PluginPrincipal::new(PluginId::new("example.native").unwrap(), "test");
        let mut registry = ServiceRegistry::<(), bool>::default();
        let grant = ServiceScope::new("example.test", Version::new(1, 0, 0), ["get"]);
        registry
            .register(principal.clone(), grant.clone(), |_| {
                std::future::ready(Ok(true))
            })
            .unwrap();
        let mut session = ServiceSession::new(principal, 1);
        let service = session.issue(&registry, grant, None).unwrap();
        let session = NativeServiceSession::new(session, Rc::new(registry));
        let codec = ServiceFrameCodec::new(64);
        let mut request = Vec::new();
        codec
            .write_call(
                &mut request,
                ServiceCallFrame {
                    handle: service,
                    operation: "get",
                    payload: b"null",
                },
            )
            .unwrap();
        let mut reply = vec![0; MAX_PACKET];
        let table = NativeServiceSession::table();
        let instance = AbiHandle::new(1);
        let mut exchange = |handle| {
            // SAFETY: the local buffers are live and disjoint throughout the callback.
            unsafe { table.exchange(handle, &request, &mut reply) }.map(<[u8]>::to_vec)
        };
        assert!(exchange(instance).is_err());
        session.enter(instance, || {
            assert!(exchange(AbiHandle::new(2)).is_err());
            let reply = exchange(instance).unwrap();
            let body = codec.decode_packet(&reply).unwrap();
            assert_eq!(
                ServiceFrameCodec::decode_reply(body).unwrap(),
                Ok(b"true".as_slice())
            );
        });
        assert!(exchange(instance).is_err());
    }
}
