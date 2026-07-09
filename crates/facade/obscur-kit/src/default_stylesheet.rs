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
//! It gives the base controls the kit's **Wireframe** look: a grayscale
//! specification-sheet aesthetic (ink `#2b2a27` on paper `#faf9f6`, hairline
//! `1.5px` borders, `2px` radii) with the full interaction-state set — a hover
//! fill shift, a `2px dashed` focus ring with offset, an inverted `:active`, a
//! dashed muted `:disabled`, and a filled `:checked` — plus `a`, `fieldset`, and
//! `table` rules. It leans only on properties and interaction pseudo-classes the
//! engine actually restyles (`:hover`, `:focus`, `:active`, `:disabled`,
//! `:checked`). Because it sits at `SystemDefault`, any author/theme sheet
//! overrides it, so themed demos keep their own look untouched.

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

/// The kit's **Wireframe** base look — a grayscale specification-sheet aesthetic
/// (ink `#2b2a27` on paper `#faf9f6`, hairline `1.5px` borders, tight `2px` radii,
/// a dashed focus ring, an inverted `:active`, a dashed muted `:disabled`, and a
/// filled `:checked`), keyed on each control's stable CSS type and restricted to
/// properties the engine renders. This is the DEFAULT look every bare control
/// gets; author/theme sheets sit above `SystemDefault` and override it freely.
///
/// Palette: ink `#2b2a27`, muted `#5a584f`, faint `#9b998f`, disabled-ink
/// `#b7b5ad`, surface `#faf9f6`, hover-fill `#e4e2db`, field-hover `#f4f2ec`,
/// disabled-fill `#f2f0ea`, rule `#d6d4cc`. Fonts are the host's (the source kit's
/// IBM Plex is not bundled), so only sizes/weights are set.
const UA_CSS: &str = "\
button {\
  padding: 11px 20px;\
  border: 1.5px solid #2b2a27;\
  border-radius: 2px;\
  background: #faf9f6;\
  color: #2b2a27;\
  font-size: 14px;\
  font-weight: 500;\
}\
button:hover {\
  background: #e4e2db;\
}\
button:focus {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 3px;\
}\
button:active {\
  background: #2b2a27;\
  color: #faf9f6;\
}\
button:disabled {\
  background: #f2f0ea;\
  color: #b7b5ad;\
  border-style: dashed;\
  border-color: #b7b5ad;\
}\
input, textarea {\
  padding: 11px 13px;\
  border: 1.5px solid #2b2a27;\
  border-radius: 2px;\
  background: #faf9f6;\
  color: #2b2a27;\
  font-size: 14px;\
}\
/* A native `<input type=text>` is SINGLE-LINE: it never wraps (long text scrolls /\
   clips). `textarea` keeps the initial `white-space: normal` so it wraps. This is\
   the only property that differs between the two — the box look is shared above. */\
input {\
  white-space: nowrap;\
}\
input:hover, textarea:hover {\
  background: #f4f2ec;\
}\
input:focus, textarea:focus {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 2px;\
}\
input:disabled, textarea:disabled {\
  border-style: dashed;\
  border-color: #b7b5ad;\
  background: #f2f0ea;\
  color: #b7b5ad;\
}\
/* checkbox / radio: the control element lays the indicator + label out in a\
   row and carries the text-level defaults; the boxed/dot indicator itself is a\
   sub-part the widget emits (`.checkbox-box` / `.radio-dot`) and this sheet\
   fills — a bare type rule would box the whole labelled control, not the\
   indicator. Indicator state is a compound class (`.checkbox-box.checked`) or a\
   descendant of the control's pseudo-class (`checkbox:hover .checkbox-box`). */\
checkbox, radio {\
  display: flex;\
  align-items: center;\
  gap: 9px;\
  color: #2b2a27;\
  font-size: 14px;\
}\
checkbox:focus, radio:focus {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 2px;\
}\
checkbox:disabled, radio:disabled {\
  color: #b7b5ad;\
}\
.checkbox-box {\
  display: flex;\
  align-items: center;\
  justify-content: center;\
  width: 20px;\
  height: 20px;\
  border: 1.5px solid #2b2a27;\
  border-radius: 2px;\
  background: #faf9f6;\
  color: #2b2a27;\
  font-size: 15px;\
}\
checkbox:hover .checkbox-box {\
  background: #e4e2db;\
}\
checkbox:focus .checkbox-box {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 2px;\
}\
.checkbox-box.checked {\
  background: #2b2a27;\
  color: #faf9f6;\
}\
.checkbox-box.indeterminate {\
  background: #2b2a27;\
}\
.checkbox-bar {\
  width: 10px;\
  height: 2px;\
  background: #faf9f6;\
}\
checkbox:disabled .checkbox-box {\
  border-style: dashed;\
  border-color: #b7b5ad;\
  background: #f2f0ea;\
  color: #b7b5ad;\
}\
.radio-dot {\
  display: flex;\
  align-items: center;\
  justify-content: center;\
  width: 20px;\
  height: 20px;\
  border: 1.5px solid #2b2a27;\
  border-radius: 10px;\
  background: #faf9f6;\
}\
radio:hover .radio-dot {\
  background: #e4e2db;\
}\
radio:focus .radio-dot {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 2px;\
}\
.radio-fill {\
  width: 10px;\
  height: 10px;\
  border-radius: 5px;\
  background: #2b2a27;\
}\
radio:disabled .radio-dot {\
  border-style: dashed;\
  border-color: #b7b5ad;\
  background: #f2f0ea;\
}\
/* The select is a vertical column: a `.select-header` row (current label + arrow)\
   and, while open, the `.select-list` popup below it. The box look (border, radius,\
   fill) is on the outer; the interior padding is on the header. */\
select {\
  display: flex;\
  flex-direction: column;\
  border: 1.5px solid #2b2a27;\
  border-radius: 2px;\
  background: #faf9f6;\
  color: #2b2a27;\
  font-size: 14px;\
}\
.select-header {\
  display: flex;\
  align-items: center;\
  justify-content: space-between;\
  gap: 10px;\
  padding: 11px 13px;\
}\
select:hover {\
  background: #e4e2db;\
}\
select:focus {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 2px;\
}\
select:disabled {\
  border-style: dashed;\
  border-color: #b7b5ad;\
  background: #f2f0ea;\
  color: #b7b5ad;\
}\
.select-arrow {\
  color: #2b2a27;\
}\
select:disabled .select-arrow {\
  color: #b7b5ad;\
}\
/* The open dropdown popup: the option list rendered as a vertical stack of rows\
   directly BELOW the header, in normal flow (a positioned popup subtree renders\
   unreliably here). It grows the control's box downward. A hairline rule separates\
   it from the header; each row keeps a fixed height (`flex-shrink: 0` so the column\
   never squashes the rows to their text). */\
.select-list {\
  display: flex;\
  flex-direction: column;\
  border-top: 1.5px solid #2b2a27;\
  background: #faf9f6;\
}\
.select-option {\
  display: flex;\
  align-items: center;\
  flex-shrink: 0;\
  height: 38px;\
  padding: 0 13px;\
  color: #2b2a27;\
  font-size: 14px;\
}\
.select-option:hover {\
  background: #e4e2db;\
}\
.select-option.selected {\
  background: #2b2a27;\
  color: #faf9f6;\
}\
.select-option.disabled {\
  color: #b7b5ad;\
}\
a {\
  color: #2b2a27;\
  border-bottom: 1.5px solid #2b2a27;\
}\
a:hover {\
  color: #000000;\
  font-weight: 600;\
}\
a:disabled {\
  color: #b7b5ad;\
}\
fieldset {\
  border: 1.5px solid #2b2a27;\
  border-radius: 3px;\
  padding: 22px 20px;\
}\
table {\
  border: 1.5px solid #2b2a27;\
  font-size: 13px;\
}\
th {\
  background: #2b2a27;\
  color: #faf9f6;\
  padding: 10px 14px;\
  font-size: 12px;\
}\
td {\
  padding: 11px 14px;\
  border-bottom: 1px solid #d6d4cc;\
  color: #2b2a27;\
}\
tr:hover {\
  background: #e4e2db;\
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
}
