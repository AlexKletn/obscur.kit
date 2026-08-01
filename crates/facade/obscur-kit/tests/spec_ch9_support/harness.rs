#![doc = include_str!("harness.doc/_module.md")]
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

#[doc = include_str!("harness.doc/compile_full.md")]
#[must_use]
pub fn compile_full(css: &str) -> CompiledStylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed)
}

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    compile_full(css).matchable
}

#[doc = include_str!("harness.doc/unsupported.md")]
#[must_use]
pub fn unsupported(css: &str) -> Vec<UnsupportedCssFeature> {
    compile_full(css).unsupported
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/runtime_id.md")]
#[must_use]
pub fn runtime_id(id: u64, w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(id), Size::new(w, h))
}

#[doc = include_str!("harness.doc/drive_sheets.md")]
#[must_use]
pub fn drive_sheets(w: f32, h: f32, sheets: &[Stylesheet], v: &View) -> UiRuntime {
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, sheets);
    rt
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    drive_sheets(w, h, std::slice::from_ref(&sheet), v)
}

#[doc = include_str!("harness.doc/at.md")]
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

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/color_of.md")]
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

#[doc = include_str!("harness.doc/assert_color.md")]
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

#[doc = include_str!("harness.doc/CapturedDiagnostics.md")]
#[derive(Debug, Clone, Default)]
pub struct CapturedDiagnostics(Arc<Mutex<Vec<Diagnostic>>>);

impl CapturedDiagnostics {
    #[doc = include_str!("harness.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("harness.doc/sink.md")]
    #[must_use]
    pub fn sink(&self) -> Box<dyn DiagnosticSink> {
        Box::new(Self(Arc::clone(&self.0)))
    }

    #[doc = include_str!("harness.doc/all.md")]
    #[must_use]
    pub fn all(&self) -> Vec<Diagnostic> {
        self.0.lock().expect("diagnostics mutex").clone()
    }

    #[doc = include_str!("harness.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.lock().expect("diagnostics mutex").len()
    }

    #[doc = include_str!("harness.doc/is_empty.md")]
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

#[doc = include_str!("harness.doc/workspace_root.md")]
#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .expect("the workspace root above crates/facade/obscur-kit")
        .to_path_buf()
}

#[doc = include_str!("harness.doc/rust_sources.md")]
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

#[doc = include_str!("harness.doc/public_api_lines.md")]
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

#[doc = include_str!("harness.doc/Shared.md")]
#[derive(Debug)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Shared<T> {
    #[doc = include_str!("harness.doc/new_2.md")]
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    #[doc = include_str!("harness.doc/set.md")]
    pub fn set(&self, value: T) {
        *self.0.borrow_mut() = value;
    }
}

impl<T: Clone> Shared<T> {
    #[doc = include_str!("harness.doc/get.md")]
    pub fn get(&self) -> T {
        self.0.borrow().clone()
    }
}
