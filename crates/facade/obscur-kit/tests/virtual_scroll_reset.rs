#![doc = include_str!("virtual_scroll_reset.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("virtual_scroll_reset.doc/LIST.md")]
const LIST: InternedSymbol = InternedSymbol::from_raw(0x9201);
#[doc = include_str!("virtual_scroll_reset.doc/ROW.md")]
const ROW: f32 = 20.0;
#[doc = include_str!("virtual_scroll_reset.doc/ITEMS.md")]
const ITEMS: usize = 10_000;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("virtual_scroll_reset.doc/view.md")]
fn view(token: u64) -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x9200))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::virtual_list(LIST, ITEMS, ROW).with_virtual_scroll_reset(token),
            ])),
    )
}

fn runtime() -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    rt.register_virtual_list(
        LIST,
        Box::new(|_index| {
            ViewNode::element(ComponentId::from_raw(0x9210))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        }),
    );
    rt
}

const SHEET: &str = ".scroller { width: 300px; height: 200px; overflow: scroll }\
                     .row { width: 300px; height: 20px }";

#[doc = include_str!("virtual_scroll_reset.doc/scroller.md")]
fn scroller(rt: &UiRuntime) -> obscur_kit::identity::NodeId {
    let root = rt.tree().root().expect("a root");
    rt.tree().get(root).expect("live").children()[0]
}

#[test]
fn changing_the_reset_token_returns_the_list_to_the_top() {
    let sheet = compile(SHEET);
    let mut rt = runtime();

    // Enter a directory (token 1) and scroll well down it.
    let _ = rt.render_now(&view(1), std::slice::from_ref(&sheet));
    rt.scroll_by(scroller(&rt), 2_000.0);
    let _ = rt.render_now(&view(1), std::slice::from_ref(&sheet));
    assert!(
        rt.virtual_window(LIST).expect("a window").start > 50,
        "the list is scrolled down before navigation"
    );

    // Enter another directory: same list, new content, new token. It must open at the
    // top, not at the previous directory's offset.
    let _ = rt.render_now(&view(2), std::slice::from_ref(&sheet));

    assert_eq!(
        rt.virtual_window(LIST).expect("a window").start,
        0,
        "a changed reset token returns the list to the top"
    );
}

#[test]
fn an_unchanged_token_keeps_the_scroll_position() {
    // The other half: a re-render with the SAME token is not a navigation, so a
    // scroll the user made must survive it. Without this the reset would fire every
    // frame and the list could never be scrolled at all.
    let sheet = compile(SHEET);
    let mut rt = runtime();

    let _ = rt.render_now(&view(1), std::slice::from_ref(&sheet));
    rt.scroll_by(scroller(&rt), 2_000.0);
    let _ = rt.render_now(&view(1), std::slice::from_ref(&sheet));
    let scrolled = rt.virtual_window(LIST).expect("a window").start;
    assert!(scrolled > 50, "the list is scrolled down");

    // Re-render with the same token — a plain reactive frame, no navigation.
    let _ = rt.render_now(&view(1), std::slice::from_ref(&sheet));

    assert_eq!(
        rt.virtual_window(LIST).expect("a window").start,
        scrolled,
        "an unchanged token leaves the scroll position untouched"
    );
}
