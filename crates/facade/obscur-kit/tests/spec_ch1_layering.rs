#![doc = include_str!("spec_ch1_layering.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use std::path::PathBuf;

use support::{rust_files, without_test_modules, workspace_root};

#[doc = include_str!("spec_ch1_layering.doc/UPWARD_CRATES.md")]
const UPWARD_CRATES: [(&str, &str); 7] = [
    ("core", "crates/foundation/obscur-geometry"),
    ("tree", "crates/domain/obscur-tree"),
    ("style", "crates/domain/obscur-style"),
    ("layout", "crates/domain/obscur-layout"),
    ("text", "crates/domain/obscur-text"),
    ("motion", "crates/domain/obscur-motion"),
    ("scene", "crates/domain/obscur-scene"),
];

#[doc = include_str!("spec_ch1_layering.doc/FORBIDDEN.md")]
const FORBIDDEN: [&str; 11] = [
    "winit::",
    "wgpu::",
    "vello::",
    "raw_window_handle::",
    "tokio::",
    "reqwest::",
    "PathBuf",
    "std::path::Path",
    "std::fs::",
    "std::net::",
    "std::os::",
];

#[doc = include_str!("spec_ch1_layering.doc/opens_public_item.md")]
fn opens_public_item(line: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with("//") {
        return false;
    }
    const OPENERS: [&str; 7] = [
        "pub fn ",
        "pub struct",
        "pub enum",
        "pub trait",
        "pub const",
        "pub static",
        "pub type",
    ];
    // A public field reads `pub name: Type,`.
    OPENERS.iter().any(|k| t.starts_with(k))
        || (t.starts_with("pub ") && t.contains(':') && !t.contains('('))
}

#[doc = include_str!("spec_ch1_layering.doc/public_signatures.md")]
fn public_signatures(source: &str) -> Vec<String> {
    let cleaned = without_test_modules(source);
    let mut out = Vec::new();
    let mut buffer: Option<(String, i32)> = None;
    for line in cleaned.lines() {
        if let Some((mut acc, mut depth)) = buffer.take() {
            acc.push(' ');
            acc.push_str(line.trim());
            depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
            if depth <= 0 {
                out.push(acc);
            } else {
                buffer = Some((acc, depth));
            }
            continue;
        }
        if !opens_public_item(line) {
            continue;
        }
        let depth = line.matches('(').count() as i32 - line.matches(')').count() as i32;
        if depth > 0 {
            buffer = Some((line.trim().to_string(), depth));
        } else {
            out.push(line.trim().to_string());
        }
    }
    if let Some((acc, _)) = buffer {
        out.push(acc);
    }
    out
}

#[doc = include_str!("spec_ch1_layering.doc/arch_01_no_platform_types_appear_in_upward_crate_signatures.md")]
#[test]
fn arch_01_no_platform_types_appear_in_upward_crate_signatures() {
    let root = workspace_root();
    let mut leaks: Vec<String> = Vec::new();
    let mut scanned = 0usize;

    for (domain, relative) in UPWARD_CRATES {
        let src: PathBuf = root.join(relative).join("src");
        assert!(
            src.is_dir(),
            "the {domain} crate must exist at {}",
            src.display()
        );
        for file in rust_files(&src) {
            let text = std::fs::read_to_string(&file).expect("readable source");
            for signature in public_signatures(&text) {
                scanned += 1;
                for token in FORBIDDEN {
                    if signature.contains(token) {
                        leaks.push(format!(
                            "{domain}: {} leaks `{token}` in `{}`",
                            file.display(),
                            signature.trim()
                        ));
                    }
                }
            }
        }
    }

    // Guard against a vacuous pass: the scan must have seen real API surface.
    assert!(
        scanned > 1000,
        "the public-signature scan only saw {scanned} signatures across the \
         seven upward crates, which cannot be their whole API surface"
    );

    assert_eq!(
        leaks.len(),
        0,
        "§1.1 expects a count of 0 platform types in the seven upward crates' \
         public signatures; found {}:\n{}",
        leaks.len(),
        leaks.join("\n")
    );
}

#[doc = include_str!("spec_ch1_layering.doc/arch_02_seven_version_domains_are_individually_addressable.md")]
#[test]
fn arch_02_seven_version_domains_are_individually_addressable() {
    // Each of the seven domains the spec names must report its own version,
    // separately queryable. Only the ones that exist can be listed here; the
    // count is the assertion.
    let mut domains: Vec<(&str, String)> = Vec::new();

    domains.push(("rust public API", env!("CARGO_PKG_VERSION").to_string()));

    let ir = obscur_kit::ui_ir::version::UiIrVersion::CURRENT;
    domains.push(("UI IR", format!("{}.{}", ir.major(), ir.minor())));

    let protocol = obscur_kit::devtools_protocol::message::ProtocolVersion::CURRENT;
    domains.push((
        "diagnostics protocol",
        format!("{}.{}", protocol.major, protocol.minor),
    ));

    // "supported CSS subset", "resource manifests", "plugin contracts" and
    // "extension/provider contracts" have no reachable version value.

    assert!(
        domains.len() > 1,
        "at least one domain other than the Rust API reports a version, or there \
         is no independence to test: {domains:?}"
    );

    // No domain's reported value may simply be the kit crate's own version.
    let kit = env!("CARGO_PKG_VERSION");
    for (name, value) in domains.iter().skip(1) {
        assert_ne!(
            value, kit,
            "the {name} version must not be the kit's CARGO_PKG_VERSION"
        );
    }
}

#[doc = include_str!("spec_ch1_layering.doc/arch_03_a_headless_only_consumer_links_no_gpu_or_window_backend.md")]
#[test]
fn arch_03_a_headless_only_consumer_links_no_gpu_or_window_backend() {
    let manifest = workspace_root()
        .join("crates/facade/obscur-kit/Cargo.toml")
        .canonicalize()
        .expect("the umbrella crate's manifest");
    let text = std::fs::read_to_string(&manifest).expect("readable manifest");

    let features = text
        .split("[features]")
        .nth(1)
        .and_then(|s| s.split("\n[").next())
        .expect("a [features] section");

    // The default set must pull in no GPU backend and no window system.
    let default_set = features
        .split("default = [")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .expect("a default feature set");
    for backend in ["render-vello", "window-winit"] {
        assert!(
            !default_set.contains(backend),
            "`{backend}` is in the default feature set, so a default consumer \
             links a backend it never asked for"
        );
    }

    // And each backend dependency must be optional, so the feature is what
    // gates linkage rather than dead-code elimination.
    for dep in [
        "obscur-render-vello",
        "obscur-window-winit",
        "obscur-render-cpu",
    ] {
        let line = text
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{dep} = ")))
            .unwrap_or_else(|| panic!("a dependency entry for {dep}"));
        assert!(
            line.contains("optional = true"),
            "`{dep}` must be an optional dependency; found `{line}`"
        );
    }
}
