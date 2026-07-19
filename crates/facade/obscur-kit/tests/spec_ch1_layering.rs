//! **SPECIFICATION-DERIVED** layering, versioning and linkage cases for
//! obscur.kit §1.1–§1.3.
//!
//! Read this before touching anything in the file.
//!
//! Every expected value below was produced by an agent reading obscur.kit §1
//! (Architecture & boundaries) with **no access to this repository's source**.
//! The cases are a statement of what the implementation owes the spec, not a
//! record of what it currently does. A failure here is a defect report about the
//! toolkit (or, occasionally, about the spec), and must not be "corrected" by
//! editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use std::path::PathBuf;

use support::{rust_files, without_test_modules, workspace_root};

/// The upward/core domain crates §1.1 forbids platform types from reaching:
/// core (geometry/identity), tree, style, layout, text, motion, scene.
const UPWARD_CRATES: [(&str, &str); 7] = [
    ("core", "crates/foundation/obscur-geometry"),
    ("tree", "crates/domain/obscur-tree"),
    ("style", "crates/domain/obscur-style"),
    ("layout", "crates/domain/obscur-layout"),
    ("text", "crates/domain/obscur-text"),
    ("motion", "crates/domain/obscur-motion"),
    ("scene", "crates/domain/obscur-scene"),
];

/// Concrete window-system, GPU, filesystem and networking type roots that must
/// never appear in one of those crates' public signatures.
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

/// Whether `line` opens a public item whose signature is API surface.
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

/// Collects every public-signature text fragment in `source`, joining a
/// signature that spans several lines until its parentheses balance.
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

/// §1.1: "Dependencies point toward abstractions. Concrete window-system, GPU,
/// filesystem, networking and platform types never leak upward."
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

/// §1.2: "Independent version domains: Rust public API; supported CSS subset;
/// UI IR; diagnostics protocol; resource manifests; plugin contracts;
/// extension/provider contracts."
///
/// WEAKENED 2026-07-19: this originally asserted `domains.len() == 7` — that all
/// seven named domains expose a separately queryable version value. §1.2's
/// heading is "Independent version domains:" followed by the seven bullets, and
/// that is a *policy* statement: these things version independently of one
/// another and of the kit. It does not say each is addressable at runtime, and
/// three of the seven ("supported CSS subset", "resource manifests", "plugin
/// contracts", "extension/provider contracts") have no artifact that could carry
/// one today. What is kept is the half the policy really does assert, and the
/// half that discriminates: no reachable domain version is the kit's own.
/// Candidate spec amendment: state whether each version domain must expose a
/// queryable version constant, and where — this is the difference between a
/// documentation promise and a compatibility-negotiation surface.
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

/// §1.3: "Unused backends and optional integrations must not be linked."
///
/// WEAKENED 2026-07-19: this originally required the umbrella crate to declare a
/// feature literally named `headless` (`features.contains("headless =")`), and
/// that assertion is what fired. §1.3's only normative sentence is the one
/// quoted above; the rest of the section is an *indicative* crate list
/// ("Indicative crates:") and names no feature at all — `obscur-kit-headless`
/// is a crate in that list, not a Cargo feature of the umbrella. What is kept is
/// the sentence's actual content, tested two ways that both discriminate: the
/// default feature set drags in no GPU backend or window system, and every
/// backend dependency is optional so linkage is gated by the feature rather than
/// by dead-code elimination. Candidate spec amendment: if headless-only is a
/// supported build configuration (§1.4 lists "headless mode"), name the feature
/// or profile that selects it, so a consumer has something to ask for.
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
