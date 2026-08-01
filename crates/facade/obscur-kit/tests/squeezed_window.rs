#![doc = include_str!("squeezed_window.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewNode};

const CSS: &str = "\
.safe       { background-color: rgba(13, 16, 23, 0.62); width: 100%; height: 100%; \
              display: flex; flex-direction: column; \
              justify-content: center; align-items: center; \
              padding-top: env(safe-area-inset-top); \
              padding-bottom: env(safe-area-inset-bottom); \
              padding-left: env(safe-area-inset-left); \
              padding-right: env(safe-area-inset-right); }\n\
.app        { flex: 1; max-width: 460px; max-height: 780px; \
              display: flex; flex-direction: column; gap: 10px; padding: 12px; }\n\
.tape       { flex: 3; background-color: rgba(20, 25, 38, 0.55); border-radius: 14px; \
              display: flex; flex-direction: column; overflow: scroll; }\n\
.tape-row   { height: 30px; display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; \
              padding-left: 14px; padding-right: 14px; overflow: hidden; }\n\
.tape-row:hover { background-color: rgba(27, 34, 51, 0.75); }\n\
.tape-text  { color: #7c89ad; font-size: 15px; white-space: nowrap; \
              overflow: hidden; text-overflow: ellipsis; }\n\
.tape-empty { color: #3f4a66; font-size: 15px; }\n\
.display    { flex: 2; background-color: rgba(20, 25, 38, 0.55); border-radius: 14px; \
              display: flex; flex-direction: column; justify-content: center; \
              padding-left: 18px; padding-right: 18px; }\n\
.pending-row{ height: 26px; display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; }\n\
.value-row  { height: 52px; display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; }\n\
.pending    { color: #6b7799; font-size: 18px; }\n\
.value      { color: #f2f5ff; font-size: 40px; }\n\
.pad        { flex: 8; display: flex; flex-direction: column; gap: 10px; }\n\
.krow       { flex: 1; display: flex; flex-direction: row; gap: 10px; }\n\
.key        { flex: 1; background-color: rgba(28, 34, 51, 0.72); border-radius: 14px; \
              color: #eef2ff; font-size: 26px; \
              display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: rgba(38, 46, 69, 0.85); }\n\
.key:active { background-color: rgba(67, 80, 122, 0.95); }\n\
.wide       { flex: 2; }\n\
.fn         { background-color: rgba(22, 27, 41, 0.66); color: #b9c4e0; font-size: 20px; }\n\
.fn:active  { background-color: rgba(51, 61, 92, 0.95); }\n\
.op         { background-color: rgba(35, 44, 73, 0.78); color: #cfe0ff; }\n\
.op:active  { background-color: rgba(70, 89, 143, 0.95); }\n\
.eq         { background-color: rgba(52, 98, 196, 0.88); color: #ffffff; }\n\
.eq:active  { background-color: rgba(91, 134, 232, 0.97); }\n\
";

fn cls(node: ViewNode, class: &str) -> ViewNode {
    node.class(class.to_owned())
}

#[doc = include_str!("squeezed_window.doc/view.md")]
fn view() -> View {
    let key = |label: &str| {
        cls(
            Container::new()
                .child(Text::new(label.to_owned()).into_view_node())
                .into_view_node(),
            "key",
        )
    };
    let row = cls(
        Container::new()
            .with_children(vec![key("7"), key("8"), key("9"), key("÷")])
            .into_view_node(),
        "krow",
    );
    let pad = cls(Container::new().child(row).into_view_node(), "pad");
    let value = cls(
        Container::new()
            .child(cls(
                Text::new("58647512".to_owned()).into_view_node(),
                "value",
            ))
            .into_view_node(),
        "value-row",
    );
    let display = cls(Container::new().child(value).into_view_node(), "display");
    let tape_row = cls(
        Container::new()
            .child(cls(
                Text::new("8932 × 6566 = 58647512".to_owned()).into_view_node(),
                "tape-text",
            ))
            .into_view_node(),
        "tape-row",
    );
    let tape = cls(Container::new().child(tape_row).into_view_node(), "tape");
    let app = cls(
        Container::new()
            .child(tape)
            .child(display)
            .child(pad)
            .into_view_node(),
        "app",
    );
    View::single(cls(Container::new().child(app).into_view_node(), "safe"))
}

fn sheet() -> Stylesheet {
    let source = CssSource::anonymous(CSS);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[test]
fn squeezing_a_window_to_nothing_and_restoring_it_completes() {
    let sheet = sheet();
    let sheets = std::slice::from_ref(&sheet);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(1200.0, 1500.0));

    // Every step a real resize passes through, including the degenerate ones a
    // compositor briefly reports while a window is being dragged to its minimum.
    for (width, height) in [
        (1200.0, 1500.0),
        (400.0, 620.0),
        (120.0, 80.0),
        (20.0, 20.0),
        (1.0, 1.0),
        (0.0, 0.0),
        (400.0, 620.0),
        (1200.0, 1500.0),
    ] {
        runtime.set_surface(Size::new(width, height), 1.0);
        let _ = runtime.render_now(&view(), sheets);
    }
}

#[doc = include_str!("squeezed_window.doc/a_container_padded_beyond_its_space_lays_out_its_growing_child.md")]
#[test]
fn a_container_padded_beyond_its_space_lays_out_its_growing_child() {
    let css = ".outer { width: 100%; height: 100%; display: flex; flex-direction: column } \
               .inner { display: flex; flex-direction: column; padding: 12px } \
               .child { flex: 8 }";
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;

    let child = cls(Container::new().into_view_node(), "child");
    let inner = cls(Container::new().child(child).into_view_node(), "inner");
    let outer = cls(Container::new().child(inner).into_view_node(), "outer");

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(20.0, 20.0));
    let _ = runtime.render_now(&View::single(outer), std::slice::from_ref(&sheet));
}
