//! The built-in **user-agent (UA) default stylesheet** — the toolkit's neutral,
//! overridable base look for the built-in controls (spec §4.14).
//!
//! This is the toolkit's analogue of a browser's UA stylesheet: a small set of
//! rules, keyed on each control's stable CSS *type* (`button`, `input`,
//! `checkbox`, `select`, …), compiled at the **lowest** cascade origin
//! ([`CascadeOrigin::SystemDefault`]). Installed on a
//! [`UiRuntime`](crate::runtime::UiRuntime) via
//! [`set_default_stylesheet`](crate::runtime::UiRuntime::set_default_stylesheet)
//! (or automatically by
//! [`KitBootstrap::install_default_stylesheet`](crate::bootstrap::KitBootstrap::install_default_stylesheet)),
//! it is matched *below* every author/theme sheet, so any author rule of the same
//! or higher specificity overrides it — exactly the browser UA-sheet contract.
//!
//! It styles the base controls with restrained, neutral defaults: a border,
//! padding and radius on `button`/`input`/`select`, an `outline`/`outline-offset`
//! focus ring, a hover shade, a dashed disabled treatment on `input`, and
//! header/cell rules for `table`. It leans only on properties and interaction
//! pseudo-classes the engine actually restyles (`:hover`, `:focus`); rules keyed
//! on `:disabled`/`:checked` are authored for completeness and take effect once
//! those states are surfaced to the matcher.

use crate::css::compiler::StylesheetCompiler;
use crate::css::parser::StylesheetParser;
use crate::css::source::CssSource;
use crate::style::cascade::CascadeOrigin;
use crate::style::matching::Stylesheet;

/// The raw CSS text of the UA default stylesheet.
///
/// Exposed so a host can inspect, extend, or re-compile it under a different
/// origin. The compiled form is [`ua_stylesheet`].
#[must_use]
pub const fn ua_css() -> &'static str {
    UA_CSS
}

/// Compiles the UA default stylesheet into a runtime-matchable [`Stylesheet`] at
/// the [`CascadeOrigin::SystemDefault`] origin — the lowest in the cascade, so any
/// author/theme declaration overrides it (§4.14).
///
/// Hand the result to
/// [`UiRuntime::set_default_stylesheet`](crate::runtime::UiRuntime::set_default_stylesheet)
/// (or let
/// [`KitBootstrap::install_default_stylesheet`](crate::bootstrap::KitBootstrap::install_default_stylesheet)
/// do it) to give an app the toolkit's overridable base component look.
#[must_use]
pub fn ua_stylesheet() -> Stylesheet {
    let source = CssSource::anonymous(UA_CSS);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new()
        .with_origin(CascadeOrigin::SystemDefault)
        .compile(&parsed)
        .matchable
}

/// The neutral base look, keyed on each control's stable CSS type. Kept small but
/// real, and restricted to properties the engine renders.
const UA_CSS: &str = "\
button {\
  padding: 6px 14px;\
  border: 1px solid #b8b5ad;\
  border-radius: 6px;\
  background: #f4f2ee;\
  color: #2b2a27;\
  font-size: 14px;\
}\
button:hover {\
  background: #e9e6df;\
  border-color: #a5a29a;\
}\
button:focus {\
  outline: 2px solid #4a7dff;\
  outline-offset: 2px;\
}\
button:disabled {\
  background: #f0efec;\
  color: #a5a29a;\
  border-color: #d7d4cd;\
}\
input, textarea {\
  padding: 6px 8px;\
  border: 1px solid #b8b5ad;\
  border-radius: 4px;\
  background: #ffffff;\
  color: #2b2a27;\
  font-size: 14px;\
}\
input:focus, textarea:focus {\
  outline: 2px solid #4a7dff;\
  outline-offset: 1px;\
  border-color: #4a7dff;\
}\
input:disabled, textarea:disabled {\
  border-style: dashed;\
  background: #f4f2ee;\
  color: #a5a29a;\
}\
checkbox, radio {\
  padding: 2px 4px;\
  color: #2b2a27;\
  font-size: 14px;\
}\
checkbox:focus, radio:focus {\
  outline: 2px solid #4a7dff;\
  outline-offset: 2px;\
}\
checkbox:checked, radio:checked {\
  color: #1f6f3d;\
}\
select {\
  padding: 6px 10px;\
  border: 1px solid #b8b5ad;\
  border-radius: 4px;\
  background: #ffffff;\
  color: #2b2a27;\
  font-size: 14px;\
}\
select:focus {\
  outline: 2px solid #4a7dff;\
  outline-offset: 2px;\
}\
a {\
  color: #2f5fd0;\
}\
fieldset {\
  border: 1px solid #d7d4cd;\
  border-radius: 6px;\
  padding: 10px;\
}\
table {\
  border: 1px solid #d7d4cd;\
  font-size: 14px;\
}\
th {\
  background: #f4f2ee;\
  color: #2b2a27;\
  padding: 6px 10px;\
  border: 1px solid #d7d4cd;\
}\
td {\
  padding: 6px 10px;\
  border: 1px solid #e5e2db;\
  color: #2b2a27;\
}\
tr:hover {\
  background: #f7f6f2;\
}\
img {\
  display: block;\
}\
";

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
        assert!(has_button_bg, "UA sheet must style `button` with a background");
    }

    #[test]
    fn ua_css_compiles_without_syntax_errors() {
        // The authored UA CSS must parse cleanly (a malformed default sheet would
        // silently drop rules). Unsupported-but-recognised properties are allowed.
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
    fn button_background_is_the_neutral_base_fill() {
        // Pin the concrete UA button background so the cascade-override proof (in
        // the runtime tests) has a known value to assert the author sheet beats.
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
                0xf4 as f32 / 255.0,
                0xf2 as f32 / 255.0,
                0xee as f32 / 255.0,
                1.0
            )))
        );
    }
}
