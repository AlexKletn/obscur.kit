#![doc = include_str!("spec_ch9_api_surface.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use support::{public_api_lines, workspace_root};

#[doc = include_str!("spec_ch9_api_surface.doc/CORE_CRATES.md")]
const CORE_CRATES: &[&str] = &[
    "crates/domain/obscur-tree",
    "crates/domain/obscur-style",
    "crates/domain/obscur-layout",
    "crates/domain/obscur-text",
    "crates/domain/obscur-scene",
    "crates/domain/obscur-view",
];

#[doc = include_str!("spec_ch9_api_surface.doc/no_core_public_signature_mentions_a_platform_type.md")]
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

#[doc = include_str!("spec_ch9_api_surface.doc/no_public_api_dispatches_or_subscribes_by_string_event_name.md")]
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

#[doc = include_str!("spec_ch9_api_surface.doc/no_core_dependency_graph_contains_an_http_tls_or_socket_crate.md")]
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
