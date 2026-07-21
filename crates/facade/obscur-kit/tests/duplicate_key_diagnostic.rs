//! Reusing one key across siblings must be *reported*, not merely slow.
//!
//! Keyed identity assumes a key names one child (§2.4). When several claim it the
//! matcher keeps the first and the rest find no partner: they are inserted as new
//! while their predecessors are removed, every frame. The UI still looks right — it
//! is only an order of magnitude slower — so without a diagnostic an author has no
//! way to connect the symptom to the cause. This was found the hard way: a probe
//! blanked the string a `<For>` used as its key, and the resulting churn read as a
//! layout pathology in the engine for several rounds of investigation.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink, DiagnosticSubsystem};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId, StableKey};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

/// Collects everything reported, for the assertions below.
#[derive(Clone, Default)]
struct Collected(Rc<RefCell<Vec<Diagnostic>>>);

impl DiagnosticSink for Collected {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A list of `count` rows; `distinct` decides whether their keys differ.
fn list(count: u64, distinct: bool) -> View {
    let rows: Vec<ViewNode> = (0..count)
        .map(|i| {
            let key = if distinct {
                format!("row-{i}")
            } else {
                "same".to_owned()
            };
            ViewNode::element(ComponentId::from_raw(0x8100 + i))
                .with_key(StableKey::from(key.as_str()))
        })
        .collect();
    View::single(
        ViewNode::element(ComponentId::from_raw(0x8000))
            .with_children(ViewFragment::from_nodes(rows)),
    )
}

/// Renders `view` twice (the second frame is the one that reconciles against a
/// retained tree) and returns everything reported.
fn diagnostics_for(view: &View) -> Vec<Diagnostic> {
    let sheet = compile("* { width: 10px; height: 10px }");
    let sheets = std::slice::from_ref(&sheet);
    let sink = Collected::default();
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    rt.set_diagnostic_sink(Box::new(sink.clone()));
    let _ = rt.render_now(view, sheets);
    let _ = rt.render_now(view, sheets);
    let all = sink.0.borrow().clone();
    all
}

#[test]
fn reusing_one_key_across_siblings_is_reported() {
    let reported = diagnostics_for(&list(5, false));
    let duplicates: Vec<&Diagnostic> = reported
        .iter()
        .filter(|d| d.code.as_str() == "tree.duplicate_key")
        .collect();

    assert!(
        !duplicates.is_empty(),
        "five siblings sharing one key must be reported, got {reported:?}"
    );
    let first = duplicates[0];
    assert_eq!(
        first.subsystem,
        DiagnosticSubsystem::Tree,
        "the fault belongs to the tree subsystem"
    );
    assert!(
        first.context.node.is_some(),
        "the diagnostic names the parent whose children collided"
    );
    assert_ne!(
        first.code.as_str(),
        first.message.as_str(),
        "the stable code is machine-readable, not the human message"
    );
}

#[test]
fn distinct_keys_report_nothing() {
    // The other half: the check must not cry wolf on a correctly keyed list.
    let reported = diagnostics_for(&list(5, true));
    assert!(
        !reported
            .iter()
            .any(|d| d.code.as_str() == "tree.duplicate_key"),
        "distinct keys are the normal case and must stay silent, got {reported:?}"
    );
}
