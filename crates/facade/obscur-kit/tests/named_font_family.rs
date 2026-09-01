#![doc = include_str!("named_font_family.doc/_module.md")]
#![cfg(all(
    feature = "render-cpu",
    feature = "text-harfrust",
    feature = "text-unicode"
))]
#![forbid(unsafe_code)]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use obscur_kit::bootstrap::{FontFaceBytes, KitBootstrap};
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::text::font::{FontAttributes, FontFamily};

use support::{compile, el, txt, view};

#[doc = include_str!("named_font_family.doc/APPLICATION_FACE.md")]
const APPLICATION_FACE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/NotoSansMono-Regular.ttf"
));

#[doc = include_str!("named_font_family.doc/run_font.md")]
fn run_font(rt: &UiRuntime) -> u32 {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|command| match command {
            PaintCommand::DrawText { run, .. } => Some(run.font()),
            _ => None,
        })
        .expect("the text painted")
}

#[doc = include_str!("named_font_family.doc/draw.md")]
fn draw(rt: &mut UiRuntime, css: &str, class: &str) -> u32 {
    let sheet = compile(css);
    let _ = rt.render_now(
        &view(vec![el(1, class, vec![txt("Ab")])]),
        std::slice::from_ref(&sheet),
    );
    run_font(rt)
}

#[doc = include_str!("named_font_family.doc/an_application_face_is_the_family_an_author_sheet_names.md")]
#[test]
fn an_application_face_is_the_family_an_author_sheet_names() {
    let boot = KitBootstrap::new().with_font(FontFaceBytes::new(
        FontFamily::named("Figtree"),
        FontAttributes::new(),
        APPLICATION_FACE,
    ));
    let set = boot.font_set();
    assert_eq!(
        set.faces().first().map(|face| face.family().name()),
        Some("Figtree"),
        "the application's own face leads the set"
    );

    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
    let installed = boot.install_text_stack(&mut rt, &set);
    let figtree = installed
        .handle(0)
        .expect("the application face has a handle");

    let named = draw(
        &mut rt,
        ".named { font-family: Figtree; } .plain { color: #000; }",
        "named",
    );
    assert_eq!(
        named, figtree,
        "a sheet naming Figtree must draw in the face the application supplied"
    );

    // The control: without the name, the same text draws in something else —
    // so the assertion above is about the *name*, not about there being only
    // one face to pick.
    let plain = draw(
        &mut rt,
        ".named { font-family: Figtree; } .plain { color: #000; }",
        "plain",
    );
    assert_ne!(
        plain, figtree,
        "an unstyled paragraph must not accidentally be the same face"
    );
}

#[doc = include_str!("named_font_family.doc/a_named_cut_is_registered_as_the_cut_it_is.md")]
#[test]
fn a_named_cut_is_registered_as_the_cut_it_is() {
    use obscur_kit::text::span::FontWeight;

    let boot = KitBootstrap::new()
        .with_font(FontFaceBytes::new(
            FontFamily::named("Figtree"),
            FontAttributes::new(),
            APPLICATION_FACE,
        ))
        .with_font(FontFaceBytes::new(
            FontFamily::named("Figtree"),
            FontAttributes::new().with_weight(FontWeight::BOLD),
            APPLICATION_FACE,
        ));
    let set = boot.font_set();
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
    let installed = boot.install_text_stack(&mut rt, &set);

    let bold = draw(
        &mut rt,
        ".b { font-family: Figtree; font-weight: 700; }",
        "b",
    );
    assert_eq!(
        Some(bold),
        installed.handle(1),
        "a bold request picks the family's bold cut, not its regular one"
    );
}

#[doc = include_str!("named_font_family.doc/fontconfig.md")]
#[cfg(feature = "font-fontconfig")]
mod fontconfig {
    use obscur_kit::bootstrap::KitBootstrap;
    use obscur_kit::font_fontconfig::source::FontconfigSource;
    use obscur_kit::geometry::Size;
    use obscur_kit::identity::RuntimeId;
    use obscur_kit::runtime::UiRuntime;
    use obscur_kit::text::font::{FontAttributes, GenericFamily};

    #[doc = include_str!("named_font_family.doc/an_installed_family_named_in_a_sheet_is_the_face_that_draws.md")]
    #[test]
    fn an_installed_family_named_in_a_sheet_is_the_face_that_draws() {
        let source = FontconfigSource::new();
        if !source.is_available() {
            eprintln!("skipped: no fontconfig on this machine");
            return;
        }
        // The host's own serif, asked for BY NAME. It is installed by
        // construction and covers Latin, and the bootstrap registered it under
        // the *generic* `serif` — never under the name an author would write,
        // which is the whole of the defect.
        let Some(serif) = source.match_generic(GenericFamily::Serif) else {
            eprintln!("skipped: fontconfig nominated no serif");
            return;
        };
        let name = serif.family().to_owned();
        let Some(by_name) = source.find_family(&name, &FontAttributes::new()) else {
            eprintln!("skipped: {name} is not listed under its own name");
            return;
        };
        let expected = std::fs::read(by_name.path()).expect("the family's own file");

        let boot = KitBootstrap::new();
        let Some(set) = boot.system_font_set() else {
            eprintln!("skipped: fontconfig nominated no loadable face");
            return;
        };
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
        let installed = boot.install_text_stack(&mut rt, &set);

        let css = format!(".named {{ font-family: \"{name}\"; }}");
        let drawn = super::draw(&mut rt, &css, "named");

        let discovered = installed.take_discovered();
        assert_eq!(
            discovered.len(),
            1,
            "the named family must be looked up and handed to the renderer once"
        );
        assert_eq!(
            discovered[0].0, drawn,
            "the run must draw in the face that lookup produced"
        );
        assert_eq!(
            discovered[0].1.as_ref(),
            expected.as_slice(),
            "and that face must be the file fontconfig lists under {name}, \
             not a substitute"
        );
    }

    #[doc = include_str!("named_font_family.doc/a_family_the_machine_does_not_have_pulls_in_nothing.md")]
    #[test]
    fn a_family_the_machine_does_not_have_pulls_in_nothing() {
        let source = FontconfigSource::new();
        if !source.is_available() {
            eprintln!("skipped: no fontconfig on this machine");
            return;
        }
        let boot = KitBootstrap::new();
        let Some(set) = boot.system_font_set() else {
            eprintln!("skipped: fontconfig nominated no loadable face");
            return;
        };
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
        let installed = boot.install_text_stack(&mut rt, &set);

        let _ = super::draw(
            &mut rt,
            ".named { font-family: \"ThisFamilyDoesNotExist12345\", serif; }",
            "named",
        );
        assert!(
            installed.take_discovered().is_empty(),
            "an absent family must register nothing — the chain falls through \
             to the generic rather than to a substituted face"
        );
    }
}
