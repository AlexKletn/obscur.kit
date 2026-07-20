//! End-to-end proof that the two historical `.okui` "bridge gaps" — real widgets
//! and `<img src>` — are closed, driven through the *real* frontend
//! (parse → compile → lower) rather than a hand-built IR node.
//!
//! Both `.okui` lowering paths resolve names through the shared
//! `obscur_ui_ir::name_binding` registry, so this file checks each gap on both:
//!
//! - the **runtime bridge** path renders an authored `<img src>` all the way to
//!   a `PaintCommand::DrawImage` (registry → reserved `SRC` → decoded pixels),
//!   and binds `<Button>` to the real `Button::COMPONENT_ID`; and
//! - the **codegen** path emits, from the same compiled IR, Rust source carrying
//!   those same real handles.
#![forbid(unsafe_code)]
#![cfg(all(
    feature = "resource-fs",
    feature = "resource-decode",
    feature = "render-cpu"
))]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::codegen::ViewCodegen;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::render::RendererId;
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{SRC, View, ViewNode};
use obscur_kit::widgets::button::Button;

/// A real 2×2 opaque-red PNG (RGB), embedded so the test needs no image-encoding
/// dependency — the fs loader reads these bytes and the decoder yields RGBA8.
const RED_2X2_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0,
    0, 0, 253, 212, 154, 115, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 0, 68, 12,
    16, 10, 0, 31, 238, 3, 253, 99, 94, 187, 91, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fresh sandbox dir holding `logo.png` (the embedded red PNG).
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-okui-gaps-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("logo.png"), RED_2X2_PNG).expect("write png");
    dir
}

/// Parses and compiles `.okui` source, returning the bridged runtime `View` and
/// the generated Rust source for the same document. Fails on any compile error.
fn compile_okui(source: &str) -> (View, String) {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to Document");
    let result = Compiler::new("test.okui").compile(&document);
    assert!(
        !result.has_errors(),
        "unexpected diagnostics: {:?}",
        result.diagnostics()
    );
    let ir = result.document().expect("a lowered UI IR document");
    let view = UiIrViewBridge::document_to_view(ir);
    let generated = ViewCodegen::document_to_view_fn(ir, "view");
    (view, generated)
}

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile_css(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[test]
fn okui_img_src_reaches_paint_on_both_paths() {
    let dir = temp_asset_dir();

    // Authored `.okui`: a real image element with a `src` and a sizing class.
    let (view, generated) = compile_okui(r#"<img src="logo.png" class="pic"/>"#);

    // 1a. Bridge path — the View element binds to the real Image primitive and
    //     its `src` reached the reserved SRC symbol (the old hash bridge dropped
    //     both, so `<img>` never painted).
    let ViewNode::Element {
        component, props, ..
    } = &view.root().nodes()[0]
    else {
        panic!("expected an element");
    };
    assert_eq!(*component, obscur_kit::primitives::Image::COMPONENT_ID);
    assert!(
        props.iter().any(|p| p.name() == SRC),
        "the `src` attribute reached the reserved SRC symbol"
    );

    // 1b. Codegen path — the emitted Rust carries the same real handles.
    assert!(
        generated.contains(&format!(
            "ComponentId::from_raw({:#x})",
            obscur_kit::primitives::Image::COMPONENT_ID.raw()
        )),
        "generated source binds <img> to the real Image ComponentId:\n{generated}"
    );
    assert!(
        generated.contains(&format!("InternedSymbol::from_raw({:#x})", SRC.raw())),
        "generated source emits the reserved SRC symbol:\n{generated}"
    );

    // 1c. Runtime end-to-end — the loaded+decoded image actually paints.
    let sheet = compile_css(".pic { width: 40px; height: 30px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(64.0, 64.0));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);
    let _renderer = CpuRenderer::new(RendererId::from_raw(2));

    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    let draw_images = snapshot
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .count();
    assert_eq!(
        draw_images, 1,
        "the `.okui` <img src> loaded, decoded and painted exactly one DrawImage"
    );
}

#[test]
fn okui_button_binds_the_real_widget_component_id_on_both_paths() {
    let (view, generated) = compile_okui(r#"<Button label="OK"/>"#);

    // Bridge path — the `<Button>` tag binds to the real widget COMPONENT_ID,
    // not a placeholder hash.
    let ViewNode::Element { component, .. } = &view.root().nodes()[0] else {
        panic!("expected an element");
    };
    assert_eq!(*component, Button::COMPONENT_ID);

    // Codegen path — the emitted Rust carries the same id.
    assert!(
        generated.contains(&format!(
            "ComponentId::from_raw({:#x})",
            Button::COMPONENT_ID.raw()
        )),
        "generated source binds <Button> to the real Button ComponentId:\n{generated}"
    );
}
