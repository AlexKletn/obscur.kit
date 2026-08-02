#![doc = include_str!("default_stylesheet.doc/_module.md")]

use crate::css::compiler::StylesheetCompiler;
use crate::css::parser::StylesheetParser;
use crate::css::source::CssSource;
use crate::style::cascade::CascadeOrigin;
use crate::style::matching::Stylesheet;

pub use obscur_theme_system::stylesheet::ua_css;

#[doc = include_str!("default_stylesheet.doc/ua_stylesheet.md")]
#[must_use]
pub fn ua_stylesheet() -> Stylesheet {
    let source = CssSource::anonymous(ua_css());
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new()
        .with_origin(CascadeOrigin::SystemDefault)
        .compile(&parsed)
        .matchable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::property::{PropertyId, PropertyValue, StyleColor};
    use crate::style::selector::SimpleSelector;

    #[test]
    fn ua_stylesheet_is_at_the_system_default_origin() {
        // The whole point: the UA sheet carries the LOWEST cascade origin, so any
        // author/theme rule overrides it.
        assert_eq!(ua_stylesheet().origin(), CascadeOrigin::SystemDefault);
    }

    #[test]
    fn ua_stylesheet_styles_the_button_type() {
        let sheet = ua_stylesheet();
        // At least one rule whose subject is the `button` type selector, carrying a
        // background declaration (the base look the cascade test asserts on).
        let has_button_bg = sheet.rules().iter().any(|rule| {
            rule.selectors().selectors().iter().any(|complex| {
                complex.subject().simples() == [SimpleSelector::Type("button".into())]
            }) && rule
                .block()
                .iter()
                .any(|d| d.property() == PropertyId::BackgroundColor)
        });
        assert!(
            has_button_bg,
            "UA sheet must style `button` with a background"
        );
    }

    #[test]
    fn ua_css_compiles_without_syntax_errors() {
        // The authored UA CSS must parse cleanly (a malformed default sheet would
        // silently drop rules). Unsupported-but-recognised properties are allowed.
        // This is the assertion the crate split makes load-bearing: the text now
        // ships from `obscur-theme-system`, which cannot reach a parser to check
        // itself, so the one crate that can is the one that must.
        let source = CssSource::anonymous(ua_css());
        let parsed = StylesheetParser::new(&source).parse();
        let compiled = StylesheetCompiler::new()
            .with_origin(CascadeOrigin::SystemDefault)
            .compile(&parsed);
        assert!(
            compiled.errors.is_empty(),
            "UA CSS has syntax errors: {:?}",
            compiled.errors
        );
    }

    #[test]
    fn input_white_space_preserves_trailing_spaces_so_the_caret_follows_them() {
        // A single-line `<input>` must use `white-space: pre`, NOT a collapsing mode
        // (`normal`/`nowrap`). Under a collapsing mode a TRAILING space hangs to zero
        // width (CSS Text 3 §7.1), so after typing a space the painted caret stays
        // put — it does not follow the space. `pre` preserves every typed space (and
        // still never wraps, keeping the field single-line), so the space keeps its
        // width and the caret advances past it.
        use crate::style::property::layout_keyword;
        let sheet = ua_stylesheet();
        let white_space = sheet
            .rules()
            .iter()
            .filter(|rule| {
                rule.selectors().selectors().iter().any(|complex| {
                    complex.subject().simples() == [SimpleSelector::Type("input".into())]
                })
            })
            .flat_map(|rule| rule.block())
            .find(|d| d.property() == PropertyId::WhiteSpace)
            .map(|d| d.value().clone());
        assert_eq!(
            white_space,
            Some(PropertyValue::Keyword(layout_keyword::WHITE_SPACE_PRE)),
            "UA `input` must be `white-space: pre` (preserving) so a trailing space \
             keeps width and the caret follows it — a collapsing mode hangs it to zero",
        );
    }

    #[test]
    fn the_placeholder_part_is_muted_with_the_wireframe_faint_ink() {
        // The placeholder (CSS `::placeholder`) is styled from the UA layer — like
        // every other control default — in the wireframe FAINT ink `#9b998f`, so an
        // empty field's hint reads as a muted hint distinct from a real value's
        // `#2b2a27` ink (and the color is NOT hard-coded in the coordinator).
        //
        // This test used to look for the same colour on a `.placeholder` CLASS
        // subject, and it passed: the sheet did carry that rule. What it did not
        // carry was any way for the rule to reach the hint — §4.16 makes the
        // placeholder a synthetic part matched through `::part()` alone, so the
        // text was painted in full ink. The claim is unchanged and the subject is
        // the one the runtime resolves; the end-to-end proof, on the painted brush
        // rather than on the sheet, is
        // `tests/ua_stylesheet_vocabulary.rs::the_placeholder_hint_is_painted_in_the_faint_ink`.
        let sheet = ua_stylesheet();
        for host in ["input", "textarea"] {
            let color = sheet
                .rules()
                .iter()
                .filter(|rule| {
                    rule.selectors().selectors().iter().any(|complex| {
                        complex.subject().simples()
                            == [
                                SimpleSelector::Type(host.into()),
                                SimpleSelector::Part("placeholder".into()),
                            ]
                    })
                })
                .flat_map(|rule| rule.block())
                .find(|d| d.property() == PropertyId::Color)
                .map(|d| d.value().clone());
            assert_eq!(
                color,
                Some(PropertyValue::Color(StyleColor::rgba(
                    0x9b as f32 / 255.0,
                    0x99 as f32 / 255.0,
                    0x8f as f32 / 255.0,
                    1.0
                ))),
                "the UA sheet mutes `{host}::part(placeholder)` to the faint ink #9b998f"
            );
        }
    }

    #[test]
    fn button_background_is_the_wireframe_paper_fill() {
        // Pin the concrete UA button background so the cascade-override proof (in
        // the runtime tests) has a known value to assert the author sheet beats.
        // The wireframe base fills the button with paper `#faf9f6`.
        let sheet = ua_stylesheet();
        let bg = sheet
            .rules()
            .iter()
            .filter(|rule| {
                rule.selectors().selectors().iter().any(|complex| {
                    complex.subject().simples() == [SimpleSelector::Type("button".into())]
                })
            })
            .flat_map(|rule| rule.block())
            .find(|d| d.property() == PropertyId::BackgroundColor)
            .map(|d| d.value().clone());
        assert_eq!(
            bg,
            Some(PropertyValue::Color(StyleColor::rgba(
                0xfa as f32 / 255.0,
                0xf9 as f32 / 255.0,
                0xf6 as f32 / 255.0,
                1.0
            )))
        );
    }

    #[test]
    fn the_text_the_facade_compiles_is_the_theme_packages_own() {
        // The move, asserted rather than assumed: `ua_css` is a re-export, so there
        // is exactly one copy of the sheet in the workspace and the facade cannot
        // drift from it by holding a second. Compare the pointer, not the bytes —
        // two equal strings would be the very failure this rules out.
        assert!(std::ptr::eq(
            ua_css(),
            obscur_theme_system::stylesheet::ua_css()
        ));
    }
}
