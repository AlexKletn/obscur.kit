//! Shutdown requests over the process control connection.

#![cfg(any(unix, windows))]

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

use obscur_plugin_executor_process::executor::ProcessShutdownChannel;

#[test]
fn reads_and_caps_the_guest_concern() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let connection = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let budget = Duration::from_secs(5);
    std::thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let (mut guest, _) = listener.accept().unwrap();
            let mut request = [0; 9];
            guest.read_exact(&mut request).unwrap();
            assert_eq!(request[0], 1);
            assert_ne!(request[1..], [0; 8]);
            guest.write_all(&[1]).unwrap();
            guest.write_all(&u64::MAX.to_le_bytes()).unwrap();
            guest.write_all(&6u32.to_le_bytes()).unwrap();
            guest.write_all(b"saving").unwrap();
        });
        let reply = ProcessShutdownChannel::new(connection, 32)
            .request(Instant::now() + budget)
            .unwrap()
            .unwrap();
        assert_eq!(reply.message.as_ref(), "saving");
        assert!(reply.requested_time <= budget);
        peer.join().unwrap();
    });
}

#[test]
fn incomplete_reply_times_out_and_closes_the_channel() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let connection = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    std::thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let (mut guest, _) = listener.accept().unwrap();
            guest.read_exact(&mut [0; 9]).unwrap();
            guest.write_all(&[1, 0]).unwrap();
            assert_eq!(guest.read(&mut [0]).unwrap(), 0);
        });
        let error = ProcessShutdownChannel::new(connection, 32)
            .request(Instant::now() + Duration::from_millis(100))
            .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
        peer.join().unwrap();
    });
}

#[cfg(unix)]
#[test]
fn supervisor_reads_the_current_executors_concern_before_final_shutdown() {
    use obscur_plugin_executor_process::executor::{ProcessExecutor, ProcessSupervisor};
    use obscur_plugin_host::{
        lifecycle::CrashRecovery,
        service::{ServiceFrameCodec, ServiceSession},
    };
    use obscur_plugin_model::identity::{PluginId, PluginPrincipal};
    use std::process::Command;

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let connection = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let executor = ProcessExecutor::spawn(
        Command::new("/bin/sh").args(["-c", "if read -r line; then exit 1; fi; exit 0"]),
        ServiceFrameCodec::new(64),
        ServiceSession::new(
            PluginPrincipal::new(PluginId::new("test.guest").unwrap(), "fixture"),
            0,
        ),
    )
    .unwrap()
    .with_shutdown_channel(ProcessShutdownChannel::new(connection, 32));
    let mut supervisor = ProcessSupervisor::new(
        executor,
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 1),
    );
    std::thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let (mut guest, _) = listener.accept().unwrap();
            guest.read_exact(&mut [0; 9]).unwrap();
            guest.write_all(&[1]).unwrap();
            guest.write_all(&1u64.to_le_bytes()).unwrap();
            guest.write_all(&4u32.to_le_bytes()).unwrap();
            guest.write_all(b"save").unwrap();
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        let concern = supervisor.shutdown_requested(deadline).unwrap().unwrap();
        assert_eq!(concern.message.as_ref(), "save");
        assert!(supervisor.shutdown_requested(deadline).unwrap().is_none());
        assert!(
            supervisor
                .shutdown_until(deadline)
                .unwrap()
                .unwrap()
                .success()
        );
        peer.join().unwrap();
    });
}
