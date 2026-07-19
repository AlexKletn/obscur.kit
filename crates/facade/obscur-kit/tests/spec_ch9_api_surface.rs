//! **SPECIFICATION-DERIVED** cases for §1.1 ("Concrete window-system, GPU,
//! filesystem, networking and platform types never leak upward."), §1.3
//! ("Unused backends and optional integrations must not be linked.") and §2.3
//! ("Events are typed. String-based method dispatch is not used.") — the
//! invariants behind the §9.6 guardrails "leak platform types into core" and
//! "introduce untyped string dispatch", and the §9.1 non-goal "arbitrary
//! networking in the core".
//!
//! These are linkage and API-surface claims, not behaviour claims, so they are
//! asserted against the repository's own manifests and public signatures.
//!
//! Every expectation was written by an agent reading the specification with **no
//! access to this repository's source**. A failure here is a defect report about
//! the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use support::{public_api_lines, workspace_root};

/// The crates §1.1 calls "core": core/tree/style/layout/text/scene, plus the
/// view layer they are addressed through.
const CORE_CRATES: &[&str] = &[
    "crates/domain/obscur-tree",
    "crates/domain/obscur-style",
    "crates/domain/obscur-layout",
    "crates/domain/obscur-text",
    "crates/domain/obscur-scene",
    "crates/domain/obscur-view",
];

/// §1.1 — "Concrete window-system, GPU, filesystem, networking and platform
/// types never leak upward." (guardrail §9.6 "leak platform types into core")
///
/// NG-PLATTYPES-01.
#[test]
fn no_core_public_signature_mentions_a_platform_type() {
    let root = workspace_root();
    let forbidden = [
        "std::path::Path",
        "std::path::PathBuf",
        "PathBuf",
        "std::fs::File",
        "std::fs::",
        "std::net::",
        "SocketAddr",
        "TcpStream",
        "RawWindowHandle",
        "wgpu::",
        "winit::",
        "vello::",
    ];
    let mut leaks = Vec::new();
    for crate_rel in CORE_CRATES {
        let dir = root.join(crate_rel);
        assert!(dir.is_dir(), "expected a core crate at {}", dir.display());
        for (file, line_no, line) in public_api_lines(&dir) {
            for needle in forbidden {
                if line.contains(needle) {
                    leaks.push(format!(
                        "{}:{line_no}: {} ({})",
                        file.display(),
                        line.trim(),
                        needle
                    ));
                }
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "core public API must name no platform type; §6.10 document/resource \
         handles are the only file-shaped input. Found {} leak(s):\n{}",
        leaks.len(),
        leaks.join("\n")
    );
}

/// §2.3 Events — "Events are typed. String-based method dispatch is not used."
///
/// NG-EVENT-TYPED-01: no public entry point in the runtime or view layers takes
/// an event or method *name* as a string.
#[test]
fn no_public_api_dispatches_or_subscribes_by_string_event_name() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for crate_rel in [
        "crates/application/obscur-runtime",
        "crates/domain/obscur-view",
        "crates/domain/obscur-interaction",
    ] {
        let dir = root.join(crate_rel);
        assert!(dir.is_dir(), "expected a crate at {}", dir.display());
        for (file, line_no, line) in public_api_lines(&dir) {
            let trimmed = line.trim();
            if !trimmed.starts_with("pub fn ") && !trimmed.starts_with("pub const fn ") {
                continue;
            }
            let names_an_event = trimmed.contains("event")
                || trimmed.contains("Event")
                || trimmed.contains("dispatch")
                || trimmed.contains("subscribe");
            let takes_a_string = trimmed.contains("&str")
                || trimmed.contains(": String")
                || trimmed.contains("impl Into<String>")
                || trimmed.contains("impl Into<Box<str>>");
            if names_an_event && takes_a_string {
                offenders.push(format!("{}:{line_no}: {trimmed}", file.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "no public API may accept an event name as a string. Found:\n{}",
        offenders.join("\n")
    );
}

/// §9.1 — "- arbitrary networking in the core;" and §1.3 "Unused backends and
/// optional integrations must not be linked."
///
/// NG-NETDEPS-01. (The per-crate resolved dependency graph is not reachable from
/// a test without shelling out to cargo — which would contend for the build lock
/// — so this asserts the two observable proxies: no HTTP/TLS/socket crate is in
/// the workspace lockfile at all, and the umbrella's default feature set names no
/// GPU or windowing backend.)
#[test]
fn no_core_dependency_graph_contains_an_http_tls_or_socket_crate() {
    let root = workspace_root();
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).expect("a workspace Cargo.lock");
    for forbidden in [
        "reqwest",
        "hyper",
        "ureq",
        "rustls",
        "native-tls",
        "openssl",
        "curl",
    ] {
        assert!(
            !lock.contains(&format!("name = \"{forbidden}\"")),
            "`{forbidden}` must not appear in the workspace dependency graph"
        );
    }

    let manifest = std::fs::read_to_string(root.join("crates/facade/obscur-kit/Cargo.toml"))
        .expect("the umbrella crate's manifest");
    let default_block = manifest
        .split("default = [")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .expect("a `default` feature list");
    for backend in ["render-vello", "window-winit", "accesskit"] {
        assert!(
            !default_block.contains(backend),
            "default features must not pull `{backend}`: {default_block}"
        );
    }

    // `tokio` may be present only as an opt-in executor adapter, and never with
    // the `net` feature.
    let tokio_adapter =
        std::fs::read_to_string(root.join("crates/adapters/obscur-task-tokio/Cargo.toml"))
            .unwrap_or_default();
    assert!(
        !tokio_adapter.contains("\"net\""),
        "no crate may enable tokio's `net` feature: {tokio_adapter}"
    );
}
