//! A virtual list builds a screenful of nodes while keeping the full extent.
//!
//! Both halves matter, and only the runtime can hold them at once. Building just the
//! window is what makes a 10,000-entry directory affordable; stating the true content
//! height is what a scroll container needs to size a scrollbar. An app can do the
//! first for itself — it cannot do the second, because to state the full height it
//! would have to build the full list, which is the cost being avoided.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The list's registered name.
const LIST: InternedSymbol = InternedSymbol::from_raw(0x9001);
/// Row height, matching the sheet.
const ROW: f32 = 20.0;
/// A deliberately large model: the point is that its size does not cost anything.
const ITEMS: usize = 10_000;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// The scroller holding the virtual list.
fn view() -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x9000))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::virtual_list(
                LIST, ITEMS, ROW,
            )])),
    )
}

/// Counts the retained descendants of the root.
fn node_count(rt: &UiRuntime) -> usize {
    rt.tree().len()
}

/// A runtime with the list registered; `built` records which indices were asked for.
fn runtime(built: &Rc<RefCell<Vec<usize>>>) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let seen = Rc::clone(built);
    rt.register_virtual_list(
        LIST,
        Box::new(move |index| {
            seen.borrow_mut().push(index);
            ViewNode::element(ComponentId::from_raw(0x9100))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        }),
    );
    rt
}

#[test]
fn only_the_visible_window_is_built() {
    let sheet = compile(
        ".scroller { width: 300px; height: 200px; overflow: scroll }\
         .row { width: 300px; height: 20px }",
    );
    let built = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&built);

    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let asked = built.borrow().len();
    assert!(
        asked > 0,
        "the registered builder must be used at all — the window is not empty"
    );
    assert!(
        asked < 100,
        "a 200px viewport over 20px rows is ~10 rows plus overscan; {asked} of \
         {ITEMS} means the list is not virtual"
    );
    assert!(
        node_count(&rt) < 100,
        "the retained tree holds the window and two spacers, not {ITEMS} rows: \
         got {}",
        node_count(&rt)
    );
}

#[test]
fn the_window_follows_the_scroll_offset() {
    let sheet = compile(
        ".scroller { width: 300px; height: 200px; overflow: scroll }\
         .row { width: 300px; height: 20px }",
    );
    let built = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&built);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let first_window = rt.virtual_window(LIST).expect("a window was realized");
    assert_eq!(first_window.start, 0, "unscrolled, the window starts at 0");

    // Scroll far down the model — a place no node currently exists for.
    built.borrow_mut().clear();
    rt.scroll_virtual_list(LIST, 4_000.0, ITEMS as f32 * ROW);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let scrolled = rt.virtual_window(LIST).expect("a window was realized");
    assert!(
        scrolled.start >= 190,
        "4000px over 20px rows lands around item 200, got {scrolled:?}"
    );
    assert!(
        built.borrow().iter().all(|&i| i >= scrolled.start - 1),
        "only the new window's rows are built after scrolling"
    );
    assert!(
        node_count(&rt) < 100,
        "still a screenful after scrolling, got {}",
        node_count(&rt)
    );
}

#[test]
fn the_content_keeps_its_full_extent() {
    // The half an app cannot state. The spacers stand in for the unbuilt rows, so
    // the scroller's content is the whole 10,000 * 20px even though ~12 rows exist.
    let sheet = compile(
        ".scroller { width: 300px; height: 200px; overflow: scroll }\
         .row { width: 300px; height: 20px }",
    );
    let built = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&built);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let scroller = rt.tree().get(root).expect("live").children()[0];
    let content: f32 = rt
        .tree()
        .get(scroller)
        .expect("live")
        .children()
        .iter()
        .filter_map(|child| rt.layout_boxes().get(child))
        .map(|b| b.size().height)
        .sum();

    let expected = ITEMS as f32 * ROW;
    assert!(
        (content - expected).abs() < 1.0,
        "the scroller's children must span the whole model ({expected}px), not just \
         the window: got {content}px"
    );
}

#[test]
fn scrolling_the_container_moves_the_window() {
    // The regression this exists for: the window was computed against a private
    // offset while the wheel moved the CONTAINER's own `scroll_top`. The two drifted,
    // so the container scrolled, the realized rows slid out of view, and the viewport
    // went blank below one clipped row. The offset must be the container's.
    let sheet = compile(
        ".scroller { width: 300px; height: 200px; overflow: scroll }\
         .row { width: 300px; height: 20px }",
    );
    let built = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&built);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let scroller = rt.tree().get(root).expect("live").children()[0];

    // Scroll the container the way the wheel does.
    rt.scroll_by(scroller, 2_000.0);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let window = rt.virtual_window(LIST).expect("a window was realized");
    assert!(
        window.start >= 90,
        "2000px over 20px rows is item ~100; the window must follow the container's \
         scroll, got {window:?}"
    );

    // And the rows that exist must be the ones the scroll landed on.
    let rows_at: Vec<usize> = built
        .borrow()
        .iter()
        .copied()
        .filter(|&i| i >= 90)
        .collect();
    assert!(
        !rows_at.is_empty(),
        "the builder must have been asked for the scrolled-to indices"
    );
}
