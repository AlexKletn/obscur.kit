//! Shared harness for the `spec_ch9_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch9_*.rs` files was written by an agent reading obscur.kit §9
//! ("Roadmap & non-goals") together with the §1–§8 clause that turns each
//! non-goal into a normative rule, **without opening any source in this
//! repository**. This module exists only to give those cases a compiling home;
//! it contains no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch9_*` test
//! is a claim about the implementation, not about the test.
#![allow(dead_code, unreachable_pub)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use obscur_kit::css::compiler::{CompiledStylesheet, StylesheetCompiler};
use obscur_kit::css::error::UnsupportedCssFeature;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

// ---------------------------------------------------------------------------
// CSS + view plumbing (same shape as the §4 harness)
// ---------------------------------------------------------------------------

/// Parses and compiles authored CSS through the real frontend, keeping the whole
/// [`CompiledStylesheet`] so a case can read its unsupported-feature reports.
#[must_use]
pub fn compile_full(css: &str) -> CompiledStylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed)
}

/// Compiles authored CSS text into a matchable stylesheet.
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    compile_full(css).matchable
}

/// The unsupported-feature reports the CSS frontend produced for `css`.
#[must_use]
pub fn unsupported(css: &str) -> Vec<UnsupportedCssFeature> {
    compile_full(css).unsupported
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A text node.
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// A runtime with a `w x h` viewport.
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

/// A runtime with a `w x h` viewport under an explicit runtime id.
#[must_use]
pub fn runtime_id(id: u64, w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(id), Size::new(w, h))
}

/// Renders one real frame of `v` under `sheets` in a fresh `w x h` runtime.
#[must_use]
pub fn drive_sheets(w: f32, h: f32, sheets: &[Stylesheet], v: &View) -> UiRuntime {
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, sheets);
    rt
}

/// Renders one real frame of `v` under a single authored stylesheet.
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    drive_sheets(w, h, std::slice::from_ref(&sheet), v)
}

/// Walks from the tree root down `path` (child indices) to a [`NodeId`].
#[must_use]
pub fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(
            i < kids.len(),
            "child index {i} out of range: node has {} children",
            kids.len()
        );
        node = kids[i];
    }
    node
}

/// The absolute border-box `(x, y, w, h)` of a node.
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

/// Asserts `actual ≈ expected` within a sub-hundredth-pixel tolerance.
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

/// A node's computed `color`.
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

/// Asserts a node's computed `color` is the 8-bit sRGB triple `(r, g, b)`.
pub fn assert_color(rt: &UiRuntime, node: NodeId, r: u8, g: u8, b: u8, what: &str) {
    let actual = color_of(rt, node).unwrap_or_else(|| panic!("{what}: no computed color"));
    let expected = StyleColor::rgba(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        1.0,
    );
    let close = |a: f32, b: f32| (a - b).abs() < 1.0 / 255.0;
    assert!(
        close(actual.r, expected.r)
            && close(actual.g, expected.g)
            && close(actual.b, expected.b)
            && close(actual.a, expected.a),
        "{what}: expected rgb({r},{g},{b}), got {actual:?}"
    );
}

// ---------------------------------------------------------------------------
// Diagnostics capture (§1.6: severity, subsystem, stable code, context)
// ---------------------------------------------------------------------------

/// A [`DiagnosticSink`] that keeps every diagnostic it is handed.
#[derive(Debug, Clone, Default)]
pub struct CapturedDiagnostics(Arc<Mutex<Vec<Diagnostic>>>);

impl CapturedDiagnostics {
    /// A fresh, empty capture buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A boxed sink writing into this buffer, for [`UiRuntime::set_diagnostic_sink`].
    #[must_use]
    pub fn sink(&self) -> Box<dyn DiagnosticSink> {
        Box::new(Self(Arc::clone(&self.0)))
    }

    /// Every diagnostic captured so far, in emission order.
    #[must_use]
    pub fn all(&self) -> Vec<Diagnostic> {
        self.0.lock().expect("diagnostics mutex").clone()
    }

    /// How many diagnostics have been captured.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.lock().expect("diagnostics mutex").len()
    }

    /// Whether nothing has been captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl DiagnosticSink for CapturedDiagnostics {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0
            .lock()
            .expect("diagnostics mutex")
            .push(diagnostic.clone());
    }
}

// ---------------------------------------------------------------------------
// Repository-shaped helpers, for the API-surface / build-graph cases
// ---------------------------------------------------------------------------

/// The workspace root (`crates/facade/obscur-kit` → three levels up).
#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .expect("the workspace root above crates/facade/obscur-kit")
        .to_path_buf()
}

/// Every `.rs` file under `dir`, recursively.
#[must_use]
pub fn rust_sources(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The lines of every public (`pub fn` / `pub struct` field / `pub const fn`)
/// item in `crate_dir`'s `src`, paired with their file and 1-based line number.
///
/// Test-only code (`#[cfg(test)]` modules) is excluded by dropping everything
/// from the first `mod tests` marker in a file onward.
#[must_use]
pub fn public_api_lines(crate_dir: &std::path::Path) -> Vec<(PathBuf, usize, String)> {
    let mut out = Vec::new();
    for file in rust_sources(&crate_dir.join("src")) {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("mod tests") {
                break;
            }
            let trimmed = line.trim_start();
            let is_public_item = trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub const fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("pub unsafe fn ")
                || trimmed.starts_with("pub ");
            if is_public_item {
                out.push((file.clone(), index + 1, line.to_string()));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Small shared cells, for the multi-runtime / binding cases
// ---------------------------------------------------------------------------

/// A shared, cloneable cell of `T` for driving root-view builders from a test.
#[derive(Debug)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Shared<T> {
    /// Wraps `value`.
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    /// Replaces the held value.
    pub fn set(&self, value: T) {
        *self.0.borrow_mut() = value;
    }
}

impl<T: Clone> Shared<T> {
    /// A clone of the held value.
    pub fn get(&self) -> T {
        self.0.borrow().clone()
    }
}
