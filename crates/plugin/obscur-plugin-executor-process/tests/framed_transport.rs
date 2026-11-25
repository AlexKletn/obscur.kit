//! End-to-end transport behavior over framed streams and child pipes.

use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

use obscur_plugin_executor_process::transport::FramedTransport;
use obscur_plugin_host::{
    capability::ServiceScope,
    service::{
        JsonServiceBridge, ServiceCallFrame, ServiceFrameCodec, ServiceRegistry, ServiceSession,
    },
};
use obscur_plugin_model::{
    identity::{PluginId, PluginPrincipal},
    manifest::Version,
};

fn principal(name: &str) -> PluginPrincipal {
    PluginPrincipal::new(PluginId::new(name).unwrap(), "installation")
}

fn scope() -> ServiceScope {
    ServiceScope::new("numbers", Version::new(1, 0, 0), ["increment"])
}

fn complete<T>(future: impl Future<Output = T>) -> T {
    match std::pin::pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test operation should finish immediately"),
    }
}

#[cfg(unix)]
#[test]
fn serves_a_child_process_over_piped_stdio() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use std::process::Command;

    let mut registry = ServiceRegistry::<u32, u32>::default();
    registry
        .register(principal("provider"), scope(), |request| async move {
            assert_eq!(request.caller.plugin().as_str(), "child");
            Ok(request.payload + 1)
        })
        .unwrap();
    let mut session = ServiceSession::new(principal("child"), 1);
    let handle = session.issue(&registry, scope(), None).unwrap();
    let codec = ServiceFrameCodec::new(64);
    let root = tempfile::tempdir().unwrap();
    let request = root.path().join("request");
    let response = root.path().join("response");
    codec
        .write_call(
            std::fs::File::create(&request).unwrap(),
            ServiceCallFrame {
                handle,
                operation: "increment",
                payload: b"41",
            },
        )
        .unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "cat \"$1\" && cat > \"$2\"", "fixture"])
        .arg(request)
        .arg(&response);
    let mut child = ProcessExecutor::spawn(&mut command, codec, session).unwrap();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    assert!(complete(child.serve_next(&bridge)).unwrap());
    assert!(child.wait().unwrap().success());
    assert!(!complete(child.serve_next(&bridge)).unwrap());
    assert!(child.try_wait().unwrap().unwrap().success());
    let reply = codec
        .read_frame(std::fs::File::open(response).unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        ServiceFrameCodec::decode_reply(&reply).unwrap(),
        Ok(b"42".as_slice())
    );
}

#[test]
fn bad_frames_and_eof_close_the_transport() {
    let registry = ServiceRegistry::<(), ()>::default();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    for (input, malformed) in [(b"\0\0\0\x01x".as_slice(), true), (b"".as_slice(), false)] {
        let mut transport = FramedTransport::new(
            ServiceFrameCodec::new(64),
            input,
            Vec::new(),
            ServiceSession::new(principal("child"), 1),
        );
        let result = complete(transport.serve_next(&bridge));
        if malformed {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
        } else {
            assert!(!result.unwrap());
        }
        assert!(!transport.is_open());
        assert!(!complete(transport.serve_next(&bridge)).unwrap());
    }
}

#[test]
fn cancellation_after_consuming_a_request_closes_the_transport() {
    let codec = ServiceFrameCodec::new(64);
    let mut registry = ServiceRegistry::<(), ()>::default();
    registry
        .register(principal("provider"), scope(), |_| std::future::pending())
        .unwrap();
    let mut session = ServiceSession::new(principal("child"), 1);
    let handle = session.issue(&registry, scope(), None).unwrap();
    let mut request = Vec::new();
    codec
        .write_call(
            &mut request,
            ServiceCallFrame {
                handle,
                operation: "increment",
                payload: b"null",
            },
        )
        .unwrap();
    let mut transport = FramedTransport::new(codec, request.as_slice(), Vec::new(), session);
    let bridge = JsonServiceBridge::new(&registry, 32, 32);

    {
        let future = transport.serve_next(&bridge);
        assert!(
            std::pin::pin!(future)
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
    }
    assert!(!transport.is_open());
    assert!(!complete(transport.serve_next(&bridge)).unwrap());
}

#[cfg(unix)]
#[test]
fn stop_terminates_and_reaps_a_running_child() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use std::process::Command;

    let mut command = Command::new("/bin/sh");
    command.args(["-c", "read -r line; exit 7"]);
    let mut child = ProcessExecutor::spawn(
        &mut command,
        ServiceFrameCodec::new(64),
        ServiceSession::new(principal("child"), 0),
    )
    .unwrap();
    assert!(child.try_wait().unwrap().is_none());
    let status = child.stop().unwrap();
    assert!(!status.success());
    assert_eq!(child.try_wait().unwrap(), Some(status));
    assert_eq!(child.stop().unwrap(), status);
}

#[cfg(unix)]
#[test]
fn stop_handle_releases_a_worker_waiting_for_a_child_frame() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use std::{process::Command, sync::Barrier};

    let mut command = Command::new("/bin/sh");
    // The shell itself waits for stdin and holds stdout open.
    command.args(["-c", "read -r line"]);
    let mut child = ProcessExecutor::spawn(
        &mut command,
        ServiceFrameCodec::new(64),
        ServiceSession::new(principal("child"), 0),
    )
    .unwrap();
    let watchdog = child.stop_handle();
    let registry = ServiceRegistry::<(), ()>::default();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    let barrier = Barrier::new(2);
    let stopped = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            barrier.wait();
            watchdog.stop().unwrap()
        });
        barrier.wait();
        assert!(!complete(child.serve_next(&bridge)).unwrap());
        worker.join().unwrap()
    });
    assert_eq!(child.try_wait().unwrap(), Some(stopped));
    assert!(!stopped.success());
}

#[cfg(unix)]
#[test]
fn supervisor_restarts_failed_processes_and_stops_after_the_failure_limit() {
    use obscur_plugin_executor_process::executor::{
        ProcessExecutor, ProcessSupervisor, ProcessSupervisorEvent,
    };
    use obscur_plugin_host::lifecycle::CrashRecovery;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    let spawn = |script: &str| {
        ProcessExecutor::spawn(
            Command::new("/bin/sh").args(["-c", script]),
            ServiceFrameCodec::new(64),
            ServiceSession::new(principal("child"), 0),
        )
    };
    let failure = || spawn("exit 7");
    let mut supervisor = ProcessSupervisor::new(
        failure().unwrap(),
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 2),
    );
    supervisor.executor_mut().unwrap().wait().unwrap();
    assert!(
        matches!(supervisor.poll(failure).unwrap(), ProcessSupervisorEvent::Exited(status) if status.code() == Some(7))
    );
    assert_eq!(
        supervisor.poll(failure).unwrap(),
        ProcessSupervisorEvent::Restarted
    );
    supervisor.executor_mut().unwrap().wait().unwrap();
    assert!(
        matches!(supervisor.poll(failure).unwrap(), ProcessSupervisorEvent::Exited(status) if status.code() == Some(7))
    );
    assert_eq!(
        supervisor.poll(|| panic!("blocked restart")).unwrap(),
        ProcessSupervisorEvent::RestartBlocked
    );

    supervisor.reset_recovery();
    assert_eq!(
        supervisor
            .poll(|| spawn("if read -r line; then exit 1; fi; exit 0"))
            .unwrap(),
        ProcessSupervisorEvent::Restarted
    );
    assert!(
        supervisor
            .shutdown_until(Instant::now() + Duration::from_secs(5))
            .unwrap()
            .unwrap()
            .success()
    );
    assert_eq!(
        supervisor.poll(|| panic!("intentional shutdown")).unwrap(),
        ProcessSupervisorEvent::Stopped
    );
}

#[cfg(unix)]
#[test]
fn supervisor_distinguishes_watchdog_timeout_and_quota_stops() {
    use obscur_plugin_executor_process::executor::{
        ProcessExecutor, ProcessSupervisor, ProcessSupervisorEvent, ProcessWatchdog,
        ProcessWatchdogOutcome,
    };
    use obscur_plugin_host::lifecycle::CrashRecovery;
    use std::{process::Command, time::Duration};

    let spawn = || {
        ProcessExecutor::spawn(
            Command::new("/bin/sh").args(["-c", "read -r line"]),
            ServiceFrameCodec::new(64),
            ServiceSession::new(principal("child"), 0),
        )
    };
    let executor = spawn().unwrap();
    let watchdog = ProcessWatchdog::arm(executor.stop_handle(), Duration::ZERO).unwrap();
    let mut supervisor = ProcessSupervisor::new(
        executor,
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 1),
    );
    let ProcessWatchdogOutcome::Expired(status) = watchdog.wait().unwrap() else {
        panic!("watchdog did not expire");
    };
    assert_eq!(
        supervisor.poll(|| panic!("unexpected launch")).unwrap(),
        ProcessSupervisorEvent::TimedOut(status)
    );
    assert_eq!(
        supervisor.poll(|| panic!("blocked launch")).unwrap(),
        ProcessSupervisorEvent::RestartBlocked
    );
    supervisor.reset_recovery();
    assert_eq!(
        supervisor.poll(spawn).unwrap(),
        ProcessSupervisorEvent::Restarted
    );
    let handle = supervisor.executor_mut().unwrap().stop_handle();
    let status = handle.stop_for_quota().unwrap();
    assert_eq!(handle.stop().unwrap(), status);
    assert_eq!(
        supervisor.poll(|| panic!("unexpected launch")).unwrap(),
        ProcessSupervisorEvent::QuotaExceeded(status)
    );
    assert_eq!(
        supervisor.poll(|| panic!("blocked launch")).unwrap(),
        ProcessSupervisorEvent::RestartBlocked
    );
}

#[cfg(unix)]
#[test]
fn process_launch_loads_committed_state_only_for_its_own_installation() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use obscur_plugin_host::storage::{
        FilesystemStateStore, PluginStorage, StateStore, StorageNamespace, StorageQuota,
    };
    use obscur_plugin_model::{manifest::VersionReq, state::StateSchema};
    use std::process::Command;

    let caller = principal("child");
    let namespace = StorageNamespace::new(caller.plugin().clone(), caller.installation());
    let directory = tempfile::tempdir().unwrap();
    let store = FilesystemStateStore::open(directory.path(), StorageQuota::new(64)).unwrap();
    let snapshot = PluginStorage::new(
        namespace.clone(),
        StateSchema::new(Version::new(1, 0, 0)),
        b"committed".to_vec(),
    );
    assert!(store.compare_exchange(None, &snapshot).unwrap());
    let codec = ServiceFrameCodec::new(64);
    let accepted = VersionReq::parse("^1").unwrap();
    assert!(matches!(
        ProcessExecutor::spawn_from_state(
            &store, &namespace, &accepted, codec,
            ServiceSession::new(principal("other"), 0), |_| panic!("must not prepare a launch"),
        ),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied
    ));
    let mut restored = ProcessExecutor::spawn_from_state(
        &store,
        &namespace,
        &accepted,
        codec,
        ServiceSession::new(caller, 0),
        |state| {
            let bytes = state.unwrap().bytes();
            let mut command = Command::new("/bin/sh");
            command.args([
                "-c",
                "test \"$1\" = committed",
                "plugin",
                std::str::from_utf8(bytes).unwrap(),
            ]);
            Ok(command)
        },
    )
    .unwrap();
    assert!(restored.wait().unwrap().success());
}

#[cfg(unix)]
#[test]
fn shutdown_allows_eof_cleanup_and_reaps_a_child_past_its_deadline() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    let spawn = |script: &str| {
        ProcessExecutor::spawn(
            Command::new("/bin/sh").args(["-c", script]),
            ServiceFrameCodec::new(64),
            ServiceSession::new(principal("child"), 0),
        )
        .unwrap()
    };
    let mut cooperative = spawn("if read -r line; then exit 1; fi; exit 0");
    assert!(
        cooperative
            .shutdown_until(Instant::now() + Duration::from_secs(5))
            .unwrap()
            .success()
    );

    let mut unresponsive = spawn("while :; do :; done");
    let error = unresponsive.shutdown_until(Instant::now()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(!unresponsive.try_wait().unwrap().unwrap().success());
}

#[cfg(unix)]
#[test]
fn watchdog_can_be_disarmed_or_stop_the_child_at_its_deadline() {
    use obscur_plugin_executor_process::executor::{
        ProcessExecutor, ProcessWatchdog, ProcessWatchdogOutcome,
    };
    use std::{process::Command, time::Duration};

    let mut command = Command::new("/bin/sh");
    command.args(["-c", "read -r line"]);
    let mut child = ProcessExecutor::spawn(
        &mut command,
        ServiceFrameCodec::new(64),
        ServiceSession::new(principal("child"), 0),
    )
    .unwrap();
    let watchdog = ProcessWatchdog::arm(child.stop_handle(), Duration::from_secs(60)).unwrap();
    assert_eq!(watchdog.disarm().unwrap(), ProcessWatchdogOutcome::Disarmed);
    assert!(child.try_wait().unwrap().is_none());

    let watchdog = ProcessWatchdog::arm(child.stop_handle(), Duration::ZERO).unwrap();
    let registry = ServiceRegistry::<(), ()>::default();
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    assert!(!complete(child.serve_next(&bridge)).unwrap());
    let status = child.try_wait().unwrap().unwrap();
    assert_eq!(
        watchdog.wait().unwrap(),
        ProcessWatchdogOutcome::Expired(status)
    );
    assert!(!status.success());
}

#[cfg(target_os = "linux")]
#[test]
fn reaping_the_group_leader_terminates_remaining_descendants() {
    use obscur_plugin_executor_process::executor::ProcessExecutor;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    let root = tempfile::tempdir().unwrap();
    let pid_file = root.path().join("descendant");
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "sleep 5 & printf '%s' \"$!\" > \"$1\"; exit 0",
            "fixture",
        ])
        .arg(&pid_file);
    let mut child = ProcessExecutor::spawn_in_process_group(
        command,
        ServiceFrameCodec::new(64),
        ServiceSession::new(principal("child"), 0),
    )
    .unwrap();
    assert!(child.wait().unwrap().success());
    let pid: u32 = std::fs::read_to_string(pid_file).unwrap().parse().unwrap();
    let stat = format!("/proc/{pid}/stat");
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match std::fs::read_to_string(&stat) {
            Ok(record)
                if record
                    .rsplit_once(") ")
                    .is_some_and(|(_, info)| info.starts_with("Z ")) =>
            {
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Ok(_) => assert!(Instant::now() < deadline, "descendant still running"),
            Err(error) => panic!("could not inspect descendant: {error}"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn handler_deadline_returns_an_error_and_keeps_the_transport_usable() {
    use obscur_plugin_host::service::ServiceWireError;

    let codec = ServiceFrameCodec::new(64);
    let mut registry = ServiceRegistry::<bool, u32>::default();
    registry
        .register(principal("provider"), scope(), |request| async move {
            if request.payload {
                std::future::pending::<()>().await;
            }
            Ok(42)
        })
        .unwrap();
    let mut session = ServiceSession::new(principal("child"), 1).with_in_flight_limit(1);
    let handle = session.issue(&registry, scope(), None).unwrap();
    let mut input = Vec::new();
    for payload in [b"true".as_slice(), b"false"] {
        codec
            .write_call(
                &mut input,
                ServiceCallFrame {
                    handle,
                    operation: "increment",
                    payload,
                },
            )
            .unwrap();
    }

    let mut output = Vec::new();
    let mut transport = FramedTransport::new(codec, input.as_slice(), &mut output, session);
    let bridge = JsonServiceBridge::new(&registry, 32, 32);
    {
        let expired = std::cell::Cell::new(false);
        let mut call = std::pin::pin!(transport.serve_next_until(&bridge, || {
            std::future::poll_fn(|_| {
                if expired.get() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        }));
        let mut context = Context::from_waker(Waker::noop());
        assert!(call.as_mut().poll(&mut context).is_pending());
        expired.set(true);
        assert!(matches!(
            call.as_mut().poll(&mut context),
            Poll::Ready(Ok(true))
        ));
    }
    assert!(transport.is_open());
    assert!(complete(transport.serve_next(&bridge)).unwrap());
    assert!(!complete(transport.serve_next(&bridge)).unwrap());
    drop(transport);

    let mut output = output.as_slice();
    let timed_out = codec.read_frame(&mut output).unwrap().unwrap();
    assert_eq!(
        ServiceFrameCodec::decode_reply(&timed_out).unwrap(),
        Err(ServiceWireError::Failed)
    );
    let successful = codec.read_frame(&mut output).unwrap().unwrap();
    assert_eq!(
        ServiceFrameCodec::decode_reply(&successful).unwrap(),
        Ok(b"42".as_slice())
    );
}
