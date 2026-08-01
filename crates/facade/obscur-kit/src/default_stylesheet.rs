#![doc = include_str!("default_stylesheet.doc/_module.md")]

use crate::css::compiler::StylesheetCompiler;
use crate::css::parser::StylesheetParser;
use crate::css::source::CssSource;
use crate::style::cascade::CascadeOrigin;
use crate::style::matching::Stylesheet;

#[doc = include_str!("default_stylesheet.doc/ua_css.md")]
#[must_use]
pub const fn ua_css() -> &'static str {
    UA_CSS
}

#[doc = include_str!("default_stylesheet.doc/ua_stylesheet.md")]
#[must_use]
pub fn ua_stylesheet() -> Stylesheet {
    let source = CssSource::anonymous(UA_CSS);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new()
        .with_origin(CascadeOrigin::SystemDefault)
        .compile(&parsed)
        .matchable
}

#[doc = include_str!("default_stylesheet.doc/UA_CSS.md")]
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
/* A native `<input type=text>` is SINGLE-LINE: `white-space: pre` preserves every\
   space the user types (no collapsing, and — crucially — no trailing-space HANG,\
   CSS Text 3 §7.1) and never wraps, so a trailing space keeps its width and the\
   caret advances past it (under a collapsing mode a trailing space hangs to zero\
   width and the caret appears stuck). Long text scrolls / clips horizontally. A\
   `<textarea>` is MULTI-LINE: `white-space: pre-wrap` preserves authored segment\
   breaks (`\n`, CSS Text 3 §4.1.1) AND wraps at the box width, so pressing Enter\
   inserts a visible newline and long lines wrap. This white-space is the only\
   property that differs between the two — the box look is shared above. */\
/* Intrinsic size: a native `<input>` defaults to ~`size=20` characters wide and a\
   `<textarea>` to ~20 cols × a few rows. The engine has no `ch` unit, so approximate\
   at the 14px UA font (~9px/char): input 180px wide (single line, clipped/scrolled),\
   textarea 240px × 72px (multi-line, wraps). An author width/height overrides this. */\
/* Overflow: a single-line `<input>` clips (`overflow: hidden`) and scrolls its run\
   HORIZONTALLY to keep the caret in view (CSS Overflow 3 §3.1). A `<textarea>` is a\
   VERTICAL scroll container (`overflow: auto`): content taller than its fixed height\
   scrolls — a wheel/scrollbar move it, and typing scrolls the caret's line into view\
   (§3.1 scroll-into-view), instead of being clipped away with no way to reach it. */\
input {\
  white-space: pre;\
  overflow: hidden;\
  width: 180px;\
}\
textarea {\
  white-space: pre-wrap;\
  overflow: auto;\
  width: 240px;\
  height: 72px;\
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
/* The placeholder overlay (CSS `::placeholder`, Pseudo-Elements L4 §4.5, shown\
   while `:placeholder-shown` holds — an empty value): a distinct, non-editable\
   sub-part the `Input` widget emits (`.placeholder`) while the field is empty, in\
   the wireframe FAINT ink `#9b998f` so it reads as a muted hint, not a real value.\
   The runtime skips this subtree when seeding / overriding the editable text. */\
.placeholder {\
  color: #9b998f;\
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
/* The disclosure arrow is a fixed-size icon: a definite width keeps it from\
   taking a fill-available flex base (auto-width flex items still fill here), so\
   `.select-header`'s `space-between` has free space to push it to the right edge. */\
.select-arrow {\
  width: 12px;\
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
        assert!(
            has_button_bg,
            "UA sheet must style `button` with a background"
        );
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
    fn placeholder_class_is_muted_with_the_wireframe_faint_ink() {
        // The `.placeholder` overlay (CSS `::placeholder`) is styled from the UA
        // layer — like every other control default — in the wireframe FAINT ink
        // `#9b998f`, so an empty field's placeholder reads as a muted hint distinct
        // from a real value's `#2b2a27` ink (and the color is NOT hard-coded in the
        // coordinator).
        let sheet = ua_stylesheet();
        let color = sheet
            .rules()
            .iter()
            .filter(|rule| {
                rule.selectors().selectors().iter().any(|complex| {
                    complex.subject().simples() == [SimpleSelector::Class("placeholder".into())]
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
            "the UA sheet mutes `.placeholder` to the wireframe faint ink #9b998f"
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
