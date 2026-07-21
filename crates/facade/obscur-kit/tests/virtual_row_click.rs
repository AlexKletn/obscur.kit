//! A click on a virtualized row resolves to the entry's **model index**.
//!
//! A virtual row cannot carry a per-row `on:click`: every row a builder emits lowers
//! to the same command symbol, so a symbol can never say which row was hit. The
//! runtime resolves it from geometry instead — the container box, its scroll offset,
//! the row extent — which is the same "only the runtime holds both halves" argument
//! the virtual list itself rests on. These tests pin that the resolved index is the
//! row under the cursor, and that it tracks the scroll offset rather than the screen
//! position.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The list's registered name.
const LIST: InternedSymbol = InternedSymbol::from_raw(0x9101);
/// Row height, matching the sheet.
const ROW: f32 = 20.0;
/// A large model — the point is that its size costs nothing.
const ITEMS: usize = 10_000;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// The scroller holding the virtual list.
fn view() -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x9100))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::virtual_list(
                LIST, ITEMS, ROW,
            )])),
    )
}

/// A runtime with the list registered and a row-click handler recording what it was
/// told; rows carry a background so a click lands on a real, hittable node.
fn runtime(recorded: &Rc<RefCell<Vec<(usize, u32)>>>) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    rt.register_virtual_list(
        LIST,
        Box::new(|_index| {
            ViewNode::element(ComponentId::from_raw(0x9110))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        }),
    );
    let sink = Rc::clone(recorded);
    rt.on_virtual_row_click(
        LIST,
        Box::new(move |index, clicks| sink.borrow_mut().push((index, clicks))),
    );
    rt
}

/// A trusted press then release at `y` — a real single click at that height.
fn click_at(rt: &mut UiRuntime, y: f32, count: usize) {
    for _ in 0..count {
        for kind in [PointerEventKind::Down, PointerEventKind::Up] {
            rt.handle_pointer(
                PointerEvent::new(
                    PointerId::MOUSE,
                    PointerDevice::Mouse,
                    kind,
                    Point::new(20.0, y),
                )
                .as_trusted(),
            );
        }
    }
}

const SHEET: &str = ".scroller { width: 300px; height: 200px; overflow: scroll; \
                     background-color: #202020 }\
                    .row { width: 300px; height: 20px; background-color: #303030 }";

#[test]
fn a_click_resolves_to_the_row_under_it() {
    let sheet = compile(SHEET);
    let recorded = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&recorded);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    // The list starts at the top of the viewport, so y = 50 is 50 / 20 = row 2.
    click_at(&mut rt, 50.0, 1);

    assert_eq!(
        recorded.borrow().as_slice(),
        &[(2, 1)],
        "a single click at y=50 over 20px rows is model index 2, click count 1"
    );
}

#[test]
fn a_double_click_is_reported_as_such() {
    let sheet = compile(SHEET);
    let recorded = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&recorded);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    click_at(&mut rt, 30.0, 2);

    // Both presses land on row 1 (30 / 20); the second is a double click. This is
    // what lets the file manager open on a double click and select on a single.
    let seen = recorded.borrow();
    assert_eq!(seen.len(), 2, "each of the two clicks fired the handler");
    assert_eq!(seen[0], (1, 1), "the first click is a single");
    assert_eq!(
        seen[1],
        (1, 2),
        "the second is a double click on the same row"
    );
}

#[test]
fn the_resolved_index_follows_the_scroll_offset() {
    // The regression this guards: the index is the entry's place in the whole model,
    // not its screen row. After scrolling, a click at the same screen height must
    // resolve to a later item — the scroll offset is added to the click's y before
    // the row extent divides it. Drop that addition and this is the only test that
    // fails, because at offset 0 the two are identical.
    let sheet = compile(SHEET);
    let recorded = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&recorded);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let scroller = rt.tree().get(root).expect("live").children()[0];

    // Scroll down by 100px — five rows — then re-render so the window follows.
    rt.scroll_by(scroller, 100.0);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    // The same screen height as the first test, now over a scrolled list: y = 50
    // plus the 100px offset is content y 150, which is row 7.
    click_at(&mut rt, 50.0, 1);

    assert_eq!(
        recorded.borrow().as_slice(),
        &[(7, 1)],
        "at scroll offset 100 a click at y=50 is model index (50 + 100) / 20 = 7"
    );
}

#[test]
fn a_short_list_that_is_not_a_scroll_container_fires_nothing() {
    // A three-row list fits the viewport whole, so it realizes no spacers and the
    // runtime tracks no container for it — there is nothing to scroll. A click in the
    // empty area below the rows therefore resolves to no row at all. (This is why the
    // resolver needs no item-count bound: only a list that overflows is ever tracked,
    // and an overflowing list fills its scrollport with rows.)
    let sheet = compile(SHEET);
    let recorded = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime(&recorded);

    let short = View::single(
        ViewNode::element(ComponentId::from_raw(0x9100))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::virtual_list(
                LIST, 3, ROW,
            )])),
    );
    let _ = rt.render_now(&short, std::slice::from_ref(&sheet));

    // In the empty area below the three 20px rows, and on a row.
    click_at(&mut rt, 120.0, 1);
    click_at(&mut rt, 30.0, 1);

    assert!(
        recorded.borrow().is_empty(),
        "a list small enough to need no scrolling is not a tracked container, so no \
         row click resolves — not the empty area, not even a row"
    );
}
