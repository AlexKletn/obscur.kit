//! Typed dispatch through the plugin host registry.

use std::{
    cell::Cell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use obscur_plugin_host::{
    capability::{CapabilityAccessError, CapabilityIssuer, ServiceScope},
    service::{ServiceDispatchError, ServiceRegistry},
};
use obscur_plugin_model::{
    identity::{PluginId, PluginPrincipal},
    manifest::Version,
};

fn principal(name: &str) -> PluginPrincipal {
    PluginPrincipal::new(PluginId::new(name).unwrap(), "installation")
}

fn scope(version: u64, operations: &[&str]) -> ServiceScope {
    ServiceScope::new(
        "resources",
        Version::new(version, 0, 0),
        operations.iter().copied(),
    )
}

fn complete<T>(future: impl Future<Output = T>) -> T {
    match std::pin::pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test provider should finish immediately"),
    }
}

#[test]
fn dispatches_the_typed_request_only_after_authorization() {
    let mut registry = ServiceRegistry::<usize, String>::default();
    let mut issuer = CapabilityIssuer::default();
    let provider = principal("provider");
    let consumer = principal("consumer");
    let calls = Rc::new(Cell::new(0));
    let invoked = Rc::clone(&calls);
    registry
        .register(
            provider.clone(),
            scope(1, &["get", "list"]),
            move |request| {
                invoked.set(invoked.get() + 1);
                async move {
                    Ok(format!(
                        "{}:{}:{}",
                        request.caller.plugin().as_str(),
                        request.operation,
                        request.payload
                    ))
                }
            },
        )
        .unwrap();
    assert_eq!(
        registry.register(provider, scope(1, &["get"]), |_| async {
            Ok(String::new())
        }),
        Err(ServiceDispatchError::AlreadyRegistered)
    );
    assert!(matches!(
        registry.issue(&mut issuer, consumer.clone(), scope(2, &["get"]), None),
        Err(ServiceDispatchError::Unsupported)
    ));
    assert!(matches!(
        registry.issue(&mut issuer, consumer.clone(), scope(1, &["write"]), None),
        Err(ServiceDispatchError::Authorization(
            CapabilityAccessError::Denied
        ))
    ));

    let handle = registry
        .issue(&mut issuer, consumer.clone(), scope(1, &["get"]), None)
        .unwrap();
    for (caller, operation) in [(&consumer, "list"), (&principal("other"), "get")] {
        assert_eq!(
            complete(registry.dispatch(&issuer, &handle, caller, operation, 9)),
            Err(ServiceDispatchError::Authorization(
                CapabilityAccessError::Denied
            ))
        );
    }
    assert_eq!(calls.get(), 0);
    assert_eq!(
        complete(registry.dispatch(&issuer, &handle, &consumer, "get", 9)),
        Ok("consumer:get:9".into())
    );
    assert_eq!(calls.get(), 1);

    let delegate = principal("delegate");
    let child = registry
        .delegate(
            &mut issuer,
            &handle,
            delegate.clone(),
            scope(1, &["get"]),
            None,
        )
        .unwrap();
    assert_eq!(handle.revoke(&mut issuer), 2);
    assert!(complete(registry.dispatch(&issuer, &child, &delegate, "get", 9)).is_err());
    assert_eq!(calls.get(), 1);
}

#[test]
fn an_unloaded_registration_cannot_be_reached_through_old_handles() {
    let mut registry = ServiceRegistry::<(), usize>::default();
    let mut issuer = CapabilityIssuer::default();
    let provider = principal("provider");
    let consumer = principal("consumer");
    registry
        .register(provider.clone(), scope(1, &["get"]), |_| async { Ok(1) })
        .unwrap();
    registry
        .register(
            principal("separate-provider"),
            scope(2, &["get"]),
            |_| async { Ok(2) },
        )
        .unwrap();
    let handle = registry
        .issue(&mut issuer, consumer.clone(), scope(1, &["get"]), None)
        .unwrap();
    let separate = registry
        .issue(&mut issuer, consumer.clone(), scope(2, &["get"]), None)
        .unwrap();
    assert_eq!(registry.unregister_provider(&provider), 1);
    assert_eq!(
        complete(registry.dispatch(&issuer, &handle, &consumer, "get", ())),
        Err(ServiceDispatchError::Unsupported)
    );
    registry
        .register(provider, scope(1, &["get"]), |_| async { Ok(3) })
        .unwrap();
    assert_eq!(
        complete(registry.dispatch(&issuer, &handle, &consumer, "get", ())),
        Err(ServiceDispatchError::Unsupported)
    );
    assert_eq!(
        complete(registry.dispatch(&issuer, &separate, &consumer, "get", ())),
        Ok(2)
    );
    let fresh = registry
        .issue(&mut issuer, consumer.clone(), scope(1, &["get"]), None)
        .unwrap();
    assert_eq!(
        complete(registry.dispatch(&issuer, &fresh, &consumer, "get", ())),
        Ok(3)
    );
}

#[test]
fn json_bridge_authorizes_before_decoding_and_bounds_both_directions() {
    use obscur_plugin_host::service::{JsonServiceBridge, ServiceWireError};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Request {
        count: usize,
    }

    let mut registry = ServiceRegistry::<Request, String>::default();
    let mut issuer = CapabilityIssuer::default();
    let consumer = principal("consumer");
    let calls = Rc::new(Cell::new(0));
    let invoked = Rc::clone(&calls);
    registry
        .register(principal("provider"), scope(1, &["get"]), move |request| {
            invoked.set(invoked.get() + 1);
            async move {
                if request.payload.count == 0 {
                    return Err(ServiceDispatchError::Rejected(
                        "private provider details".into(),
                    ));
                }
                Ok("x".repeat(request.payload.count))
            }
        })
        .unwrap();
    let handle = registry
        .issue(&mut issuer, consumer.clone(), scope(1, &["get"]), None)
        .unwrap();
    let bridge = JsonServiceBridge::new(&registry, 28, 5);

    assert_eq!(
        complete(bridge.dispatch(&issuer, &handle, &principal("other"), "get", b"not json")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(
        complete(bridge.dispatch(&issuer, &handle, &consumer, "get", b"not json")),
        Err(ServiceWireError::InvalidRequest)
    );
    assert_eq!(
        complete(bridge.dispatch(&issuer, &handle, &consumer, "get", &[b' '; 26])),
        Err(ServiceWireError::RequestTooLarge)
    );
    assert_eq!(calls.get(), 0);
    assert_eq!(
        complete(bridge.dispatch(&issuer, &handle, &consumer, "get", br#"{"count":3}"#)),
        Ok(br#""xxx""#.to_vec())
    );
    assert_eq!(
        complete(bridge.dispatch(&issuer, &handle, &consumer, "get", br#"{"count":4}"#)),
        Err(ServiceWireError::ResponseTooLarge)
    );
    let error = complete(bridge.dispatch(&issuer, &handle, &consumer, "get", br#"{"count":0}"#))
        .unwrap_err();
    assert_eq!(serde_json::to_string(&error).unwrap(), r#""failed""#);
}

#[test]
fn transport_handles_are_bound_to_the_session_and_never_reassigned() {
    use obscur_plugin_host::service::{
        JsonServiceBridge, ServiceSession, ServiceSessionError, ServiceWireError,
    };

    let mut registry = ServiceRegistry::<(), String>::default();
    registry
        .register(
            principal("provider"),
            scope(1, &["get"]),
            |request| async move { Ok(request.caller.installation().into()) },
        )
        .unwrap();

    let id = PluginId::new("same-plugin").unwrap();
    let mut first = ServiceSession::new(PluginPrincipal::new(id.clone(), "first"), 1);
    let mut second = ServiceSession::new(PluginPrincipal::new(id.clone(), "second"), 1);
    let stale = first.issue(&registry, scope(1, &["get"]), None).unwrap();
    let other = second.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 16, 16);
    assert_eq!(
        first.issue(&registry, scope(1, &["get"]), None),
        Err(ServiceSessionError::HandleLimit)
    );
    assert_eq!(
        complete(bridge.dispatch_session(&first, stale, "get", b"null")),
        Ok(br#""first""#.to_vec())
    );
    assert_eq!(
        complete(bridge.dispatch_session(&second, stale, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
    assert!(!second.release(stale));
    assert_eq!(
        complete(bridge.dispatch_session(&second, other, "get", b"null")),
        Ok(br#""second""#.to_vec())
    );

    assert!(first.release(stale));
    assert!(!first.release(stale));
    let replacement = first.issue(&registry, scope(1, &["get"]), None).unwrap();
    assert_ne!(stale, replacement);
    assert_eq!(
        complete(bridge.dispatch_session(&first, stale, "get", b"null")),
        Err(ServiceWireError::Denied)
    );

    drop(first);
    let mut reconnected = ServiceSession::new(PluginPrincipal::new(id, "first"), 1);
    let current = reconnected
        .issue(&registry, scope(1, &["get"]), None)
        .unwrap();
    assert_ne!(current, replacement);
    assert_eq!(
        complete(bridge.dispatch_session(&reconnected, replacement, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(
        complete(bridge.dispatch_session(&reconnected, current, "get", b"null")),
        Ok(br#""first""#.to_vec())
    );
}

#[test]
fn framed_calls_round_trip_through_authenticated_dispatch() {
    use obscur_plugin_host::service::{
        JsonServiceBridge, ServiceCallFrame, ServiceFrameCodec, ServiceSession, ServiceWireError,
    };

    let mut registry = ServiceRegistry::<u32, u32>::default();
    registry
        .register(
            principal("provider"),
            scope(1, &["get"]),
            |request| async move { Ok(request.payload + 1) },
        )
        .unwrap();
    let mut session = ServiceSession::new(principal("consumer"), 1);
    let handle = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    let codec = ServiceFrameCodec::new(64);

    let mut calls = Vec::new();
    for operation in ["get", "other"] {
        codec
            .write_call(
                &mut calls,
                ServiceCallFrame {
                    handle,
                    operation,
                    payload: b"41",
                },
            )
            .unwrap();
    }
    let mut calls = calls.as_slice();
    let mut replies = Vec::new();
    while let Some(body) = codec.read_frame(&mut calls).unwrap() {
        let call = ServiceFrameCodec::decode_call(&body).unwrap();
        let reply = complete(bridge.dispatch_frame(&session, call));
        codec
            .write_reply(&mut replies, reply.as_deref().map_err(|error| *error))
            .unwrap();
    }
    let mut replies = replies.as_slice();
    let first = codec.read_frame(&mut replies).unwrap().unwrap();
    assert_eq!(
        ServiceFrameCodec::decode_reply(&first).unwrap(),
        Ok(b"42".as_slice())
    );
    let second = codec.read_frame(&mut replies).unwrap().unwrap();
    assert_eq!(
        ServiceFrameCodec::decode_reply(&second).unwrap(),
        Err(ServiceWireError::Denied)
    );
    assert!(codec.read_frame(&mut replies).unwrap().is_none());
}

#[test]
fn rejects_oversized_truncated_and_malformed_frames() {
    use obscur_plugin_host::service::{ServiceCallFrame, ServiceFrameCodec};
    use std::io::ErrorKind;

    let codec = ServiceFrameCodec::new(16);
    let mut input = b"\0\0\0\x11unread".as_slice();
    assert_eq!(
        codec.read_frame(&mut input).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(input, b"unread");
    for input in [b"\0\0".as_slice(), b"\0\0\0\x02x".as_slice()] {
        assert_eq!(
            codec.read_frame(input).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
    }
    assert!(codec.read_frame(b"\0\0\0\0".as_slice()).is_err());

    let mut framed = Vec::new();
    let call = ServiceCallFrame {
        handle: 1,
        operation: "get",
        payload: b"null",
    };
    assert!(codec.write_call(&mut framed, call).is_err());
    assert!(framed.is_empty());

    let mut invalid = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0xff];
    assert!(ServiceFrameCodec::decode_call(&invalid).is_err());
    invalid[11] = b'x';
    invalid[10] = 2;
    assert!(ServiceFrameCodec::decode_call(&invalid).is_err());
    invalid[10] = 1;
    invalid[0] = 2;
    assert!(ServiceFrameCodec::decode_call(&invalid).is_err());
    assert!(ServiceFrameCodec::decode_reply(b"\x01\x01\"unknown\"").is_err());
}

#[test]
fn transport_call_budget_counts_invalid_handles_and_is_session_local() {
    use obscur_plugin_host::service::{
        JsonServiceBridge, ServiceCallLimit, ServiceSession, ServiceWireError,
    };
    use std::time::Duration;

    let mut registry = ServiceRegistry::<(), bool>::default();
    registry
        .register(principal("provider"), scope(1, &["get"]), |_| async {
            Ok(true)
        })
        .unwrap();
    let limit = ServiceCallLimit::new(1, Duration::from_secs(60)).unwrap();
    let mut first = ServiceSession::new(principal("consumer"), 1).with_call_limit(limit);
    let mut second = ServiceSession::new(principal("consumer"), 1).with_call_limit(limit);
    let first_handle = first.issue(&registry, scope(1, &["get"]), None).unwrap();
    let second_handle = second.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);

    assert_eq!(
        complete(bridge.dispatch_session(&first, second_handle, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(first.metrics().rate_limit_rejections, 0);
    assert_eq!(
        complete(bridge.dispatch_session(&first, first_handle, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(first.metrics().rate_limit_rejections, 1);
    assert_eq!(second.metrics().rate_limit_rejections, 0);
    assert_eq!(
        complete(bridge.dispatch_session(&second, second_handle, "get", b"null")),
        Ok(b"true".to_vec())
    );
    assert_eq!(
        complete(bridge.dispatch_session(&second, second_handle, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
}

#[test]
fn concurrent_call_limit_releases_capacity_on_cancellation_and_completion() {
    use obscur_plugin_host::service::{JsonServiceBridge, ServiceSession, ServiceWireError};

    let mut registry = ServiceRegistry::<bool, bool>::default();
    let calls = Rc::new(Cell::new(0));
    let invoked = Rc::clone(&calls);
    registry
        .register(principal("provider"), scope(1, &["get"]), move |request| {
            invoked.set(invoked.get() + 1);
            async move {
                if request.payload {
                    std::future::pending::<()>().await;
                }
                Ok(request.payload)
            }
        })
        .unwrap();
    let mut session = ServiceSession::new(principal("consumer"), 1)
        .with_in_flight_limit(1)
        .with_slow_call_threshold(std::time::Duration::ZERO);
    let observer = session.observer();
    let handle = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);

    {
        let mut pending = std::pin::pin!(bridge.dispatch_session(&session, handle, "get", b"true"));
        assert!(
            pending
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert_eq!(
            complete(bridge.dispatch_session(&session, handle, "get", b"false")),
            Err(ServiceWireError::Denied)
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(session.metrics().in_flight, 1);
        assert_eq!(observer.metrics(), session.metrics());
        assert_eq!(session.metrics().concurrency_limit_rejections, 1);
        assert_eq!(session.metrics().slow_calls, 0);
    }
    assert_eq!(session.metrics().in_flight, 0);
    assert_eq!(session.metrics().slow_calls, 1);

    for _ in 0..2 {
        assert_eq!(
            complete(bridge.dispatch_session(&session, handle, "get", b"false")),
            Ok(b"false".to_vec())
        );
    }
    assert_eq!(calls.get(), 3);
    assert_eq!(session.metrics().in_flight, 0);
    assert_eq!(session.metrics().concurrency_limit_rejections, 1);
    assert_eq!(session.metrics().slow_calls, 3);
    let final_metrics = session.metrics();
    drop(session);
    assert_eq!(observer.metrics(), final_metrics);
}

#[test]
fn panics_in_service_handlers_are_contained_and_release_the_session_slot() {
    use obscur_plugin_host::service::{JsonServiceBridge, ServiceSession, ServiceWireError};

    let mut registry = ServiceRegistry::<u32, u32>::default();
    registry
        .register(principal("provider"), scope(1, &["get"]), |request| {
            assert_ne!(request.payload, 1, "handler construction failed");
            async move {
                assert_ne!(request.payload, 2, "handler poll failed");
                Ok(request.payload)
            }
        })
        .unwrap();
    let consumer = principal("consumer");
    let mut issuer = CapabilityIssuer::default();
    let direct = registry
        .issue(&mut issuer, consumer.clone(), scope(1, &["get"]), None)
        .unwrap();
    for value in [1, 2] {
        assert_eq!(
            complete(registry.dispatch(&issuer, &direct, &consumer, "get", value)),
            Err(ServiceDispatchError::Panicked)
        );
    }

    let mut session = ServiceSession::new(consumer, 1).with_in_flight_limit(1);
    let handle = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 16, 16);
    for payload in [b"1", b"2"] {
        assert_eq!(
            complete(bridge.dispatch_session(&session, handle, "get", payload)),
            Err(ServiceWireError::Failed)
        );
    }
    assert_eq!(
        complete(bridge.dispatch_session(&session, handle, "get", b"3")),
        Ok(b"3".to_vec())
    );
}

#[test]
fn revoking_a_session_wakes_and_cancels_all_pending_calls() {
    use obscur_plugin_host::service::{
        JsonServiceBridge, ServiceSession, ServiceSessionError, ServiceWireError,
    };
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::Wake,
    };

    #[derive(Default)]
    struct CountWakes(AtomicUsize);

    impl Wake for CountWakes {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    let mut registry = ServiceRegistry::<(), ()>::default();
    let calls = Rc::new(Cell::new(0));
    let invoked = Rc::clone(&calls);
    registry
        .register(principal("provider"), scope(1, &["get"]), move |_| {
            invoked.set(invoked.get() + 1);
            std::future::pending()
        })
        .unwrap();
    let mut session = ServiceSession::new(principal("consumer"), 2);
    let handle = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let revoker = session.revoker();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    {
        let wakes = Arc::new(CountWakes::default());
        let waker = Waker::from(Arc::clone(&wakes));
        let mut context = Context::from_waker(&waker);
        let mut left = std::pin::pin!(bridge.dispatch_session(&session, handle, "get", b"null"));
        let mut right = std::pin::pin!(bridge.dispatch_session(&session, handle, "get", b"null"));
        assert!(left.as_mut().poll(&mut context).is_pending());
        assert!(right.as_mut().poll(&mut context).is_pending());
        assert_eq!(calls.get(), 2);
        let control = revoker.clone();
        assert!(std::thread::spawn(move || control.revoke()).join().unwrap());
        assert!(!revoker.revoke());
        assert_eq!(wakes.0.load(Ordering::Relaxed), 2);
        assert_eq!(
            left.as_mut().poll(&mut context),
            Poll::Ready(Err(ServiceWireError::Denied))
        );
        assert_eq!(
            right.as_mut().poll(&mut context),
            Poll::Ready(Err(ServiceWireError::Denied))
        );
    }
    assert_eq!(
        complete(bridge.dispatch_session(&session, handle, "get", b"null")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(
        session.issue(&registry, scope(1, &["get"]), None),
        Err(ServiceSessionError::Revoked)
    );
    assert_eq!(calls.get(), 2);
}

#[test]
fn revoking_one_transport_handle_cancels_only_its_calls() {
    use obscur_plugin_host::service::{JsonServiceBridge, ServiceSession, ServiceWireError};

    let mut registry = ServiceRegistry::<bool, bool>::default();
    registry
        .register(
            principal("provider"),
            scope(1, &["get"]),
            |request| async move {
                if request.payload {
                    std::future::pending::<()>().await;
                }
                Ok(false)
            },
        )
        .unwrap();
    let mut session = ServiceSession::new(principal("consumer"), 2).with_in_flight_limit(1);
    let first = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let second = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let revoker = session.handle_revoker(first).unwrap();
    let second_revoker = session.handle_revoker(second).unwrap();
    assert_eq!(session.active_scopes().count(), 2);
    let bridge = JsonServiceBridge::new(&registry, 32, 32);

    {
        let mut pending = std::pin::pin!(bridge.dispatch_session(&session, first, "get", b"true"));
        let mut context = Context::from_waker(Waker::noop());
        assert!(pending.as_mut().poll(&mut context).is_pending());
        assert!(revoker.revoke());
        assert_eq!(session.active_scopes().count(), 1);
        assert_eq!(
            pending.as_mut().poll(&mut context),
            Poll::Ready(Err(ServiceWireError::Denied))
        );
    }
    assert_eq!(
        complete(bridge.dispatch_session(&session, first, "get", b"false")),
        Err(ServiceWireError::Denied)
    );
    assert_eq!(
        complete(bridge.dispatch_session(&session, second, "get", b"false")),
        Ok(b"false".to_vec())
    );
    assert!(!second_revoker.is_revoked());
    assert!(session.release(first));
    assert!(session.handle_revoker(first).is_none());
    assert!(session.issue(&registry, scope(1, &["get"]), None).is_ok());
    assert!(session.revoker().revoke());
    assert_eq!(session.active_scopes().count(), 0);
    drop(session);
    assert!(second_revoker.is_revoked());
}

#[test]
fn withdrawing_a_service_registration_cancels_only_that_versions_pending_calls() {
    use obscur_plugin_host::service::{
        JsonServiceBridge, ServiceSession, ServiceSessionError, ServiceWireError,
    };

    let mut registry = ServiceRegistry::<(), bool>::default();
    let provider = principal("provider");
    registry
        .register(provider.clone(), scope(1, &["get"]), |_| {
            std::future::pending()
        })
        .unwrap();
    registry
        .register(principal("other-provider"), scope(2, &["get"]), |_| async {
            Ok(true)
        })
        .unwrap();
    let mut session = ServiceSession::new(principal("consumer"), 3).with_in_flight_limit(1);
    let old = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let other = session.issue(&registry, scope(2, &["get"]), None).unwrap();
    let revoker = registry
        .registration_revoker("resources", &Version::new(1, 0, 0))
        .unwrap();
    let (owner, registered) = registry.active_services().next().unwrap();
    assert_eq!(owner, &provider);
    assert_eq!(registered.service(), "resources");
    assert_eq!(registered.version(), &Version::new(1, 0, 0));
    assert_eq!(registered.operations().collect::<Vec<_>>(), ["get"]);
    assert_eq!(registry.active_services().count(), 2);

    {
        let bridge = JsonServiceBridge::new(&registry, 16, 16);
        let mut pending = std::pin::pin!(bridge.dispatch_session(&session, old, "get", b"null"));
        let mut context = Context::from_waker(Waker::noop());
        assert!(pending.as_mut().poll(&mut context).is_pending());
        assert!(revoker.revoke());
        let remaining: Vec<_> = registry.active_services().collect();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].1.version(), &Version::new(2, 0, 0));
        assert_eq!(
            session.active_scopes().collect::<Vec<_>>(),
            [(&scope(2, &["get"]), None)]
        );
        assert_eq!(
            pending.as_mut().poll(&mut context),
            Poll::Ready(Err(ServiceWireError::Unsupported))
        );
        assert_eq!(
            complete(bridge.dispatch_session(&session, other, "get", b"null")),
            Ok(b"true".to_vec())
        );
    }

    assert_eq!(
        session.issue(&registry, scope(1, &["get"]), None),
        Err(ServiceSessionError::Dispatch(
            ServiceDispatchError::Unsupported
        ))
    );
    assert_eq!(registry.unregister_provider(&provider), 1);
    registry
        .register(provider, scope(1, &["get"]), |_| async { Ok(false) })
        .unwrap();
    let fresh = session.issue(&registry, scope(1, &["get"]), None).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 16, 16);
    assert_eq!(
        complete(bridge.dispatch_session(&session, old, "get", b"null")),
        Err(ServiceWireError::Unsupported)
    );
    assert_eq!(
        complete(bridge.dispatch_session(&session, fresh, "get", b"null")),
        Ok(b"false".to_vec())
    );
}
