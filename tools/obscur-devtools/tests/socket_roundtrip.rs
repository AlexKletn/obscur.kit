//! End-to-end: the DevTools CLI client drives the demo host over a **real**
//! loopback socket (spec §7.6/§7.7). One process, two threads, one kernel
//! socket — exercising handshake → authorize → inspect → typed reply through the
//! unchanged endpoint and the same `WireCodec` framing a separate process uses.
#![forbid(unsafe_code)]

use std::thread;

use obscur_devtools::{cli, host};
use obscur_devtools_protocol::message::{HandshakeResponse, Inspectable, ScopeSet};
use obscur_devtools_server::transport::{ClientBound, SocketClient, SocketListener};

/// Binds a loopback host serving exactly one session with `token`/`scopes`, and
/// returns its address plus the server thread's join handle.
fn spawn_host(
    token: &'static str,
    scopes: ScopeSet,
) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        host::serve_one(&listener, host::demo_source(), token, scopes).expect("serve");
    });
    (addr, handle)
}

#[test]
fn handshake_then_inspect_returns_seeded_snapshots() {
    let (addr, handle) = spawn_host("dev", ScopeSet::all());

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    let response = cli::handshake(&mut client, "dev").expect("handshake io");
    assert!(
        matches!(response, HandshakeResponse::Accepted { .. }),
        "{response:?}"
    );

    // Inspect two targets on the SAME session; the seeded live-state comes back.
    let ClientBound::Snapshot(tree) = cli::inspect(&mut client, 1, Inspectable::Tree).expect("io")
    else {
        panic!("expected a tree snapshot");
    };
    assert_eq!(tree.target, Inspectable::Tree);
    assert_eq!(tree.payload.entity_count, 13);

    let ClientBound::Snapshot(scene) =
        cli::inspect(&mut client, 2, Inspectable::Scene).expect("io")
    else {
        panic!("expected a scene snapshot");
    };
    assert_eq!(scene.payload.entity_count, 77);
    assert!(scene.payload.description.contains("paint commands"));

    // The formatted line the CLI prints is non-empty and names the target.
    assert!(cli::format_snapshot(&scene).contains("scene"));

    drop(client);
    handle.join().expect("join");
}

#[test]
fn every_target_is_inspectable_over_the_socket() {
    let (addr, handle) = spawn_host("dev", ScopeSet::all_read());

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        cli::handshake(&mut client, "dev")
            .expect("io")
            .is_accepted()
    );

    for (index, target) in cli::all_targets().into_iter().enumerate() {
        let reply = cli::inspect(&mut client, index as u64 + 1, target).expect("io");
        let ClientBound::Snapshot(snapshot) = reply else {
            panic!("target {target:?} did not answer with a snapshot: {reply:?}");
        };
        assert_eq!(snapshot.target, target);
    }

    drop(client);
    handle.join().expect("join");
}

#[test]
fn unknown_token_is_rejected() {
    let (addr, handle) = spawn_host("secret", ScopeSet::all());

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    let response = cli::handshake(&mut client, "intruder").expect("handshake io");
    assert!(!response.is_accepted(), "an unknown token must be rejected");

    drop(client);
    handle.join().expect("join");
}

#[test]
fn read_only_session_cannot_control_but_a_subscribe_is_granted() {
    use obscur_devtools_protocol::message::Topic;

    let (addr, handle) = spawn_host("dev", ScopeSet::observer());

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        cli::handshake(&mut client, "dev")
            .expect("io")
            .is_accepted()
    );

    // observer() grants read + subscribe, so a Subscribe is granted...
    let reply = cli::subscribe(&mut client, 1, Topic::Frames).expect("io");
    let ClientBound::Subscription(subscription) = reply else {
        panic!("expected a subscription grant: {reply:?}");
    };
    assert!(subscription.granted);
    assert_eq!(subscription.topic, Topic::Frames);

    drop(client);
    handle.join().expect("join");
}
