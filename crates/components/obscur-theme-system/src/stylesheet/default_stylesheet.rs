#![doc = include_str!("default_stylesheet.doc/_module.md")]

#[doc = include_str!("default_stylesheet.doc/ua_css.md")]
#[must_use]
pub const fn ua_css() -> &'static str {
    UA_CSS
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
/* The placeholder hint (CSS `::placeholder`, Pseudo-Elements L4 §4.5, shown while\
   `:placeholder-shown` holds — an empty value), in the wireframe FAINT ink `#9b998f`\
   so it reads as a muted hint and not a real value.\
   Spelled as an EXPORTED PART, because that is what it is: §4.16 makes the\
   placeholder a synthetic, paint-only part the runtime generates from the field's\
   reserved `placeholder` attribute — `Input::view` says so outright (‘the\
   placeholder is NOT a child’), and `resolve_generated_content` resolves its colour\
   through `part_specificity_for`, which reads a `::part()` subject and nothing else.\
   This rule used to read `.placeholder { color: #9b998f }`, a CLASS, and no node the\
   toolkit builds carries that class: the hint was painted in full ink `#2b2a27`,\
   indistinguishable from a typed value, which is the one dead rule in this sheet\
   whose deadness was VISIBLE. (`.placeholder` as a class remains meaningful to the\
   runtime's editing layer — `editing_role` reads it to keep a host-materialised\
   overlay child out of the editable seed — but that node is a host's construction,\
   so it is a host's to style.) */\
input::part(placeholder), textarea::part(placeholder) {\
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
/* A disabled select's rows read muted. This used to be spelled `.select-option\
   .disabled` — a compound with a `disabled` CLASS — and no `disabled` class is\
   emitted anywhere: `Select`'s options are plain labels, and `disabled` is a\
   property of the whole control, so the rule matched nothing the toolkit builds.\
   The state the widget does have is spelled the way the arrow rule above already\
   spells it: a descendant of the disabled control. `.select-option`'s own `color`\
   is a class (0,1,0) and this is (0,2,1), so it wins on specificity rather than\
   on source order. Reachable in exactly one state, and worth knowing which:\
   `Select::open` refuses on a disabled control, so an open disabled select is one\
   that was disabled while its popup was already down — a form locking mid-\
   interaction. */\
select:disabled .select-option {\
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
/* The DRAWN controls — `progress`, `slider`, `canvas`. The three elements whose\
   interior is painted rather than built out of boxes: each is a box with a\
   `<canvas>` child naming a painter (`Progress::PAINTER`, `Slider::PAINTER`, or\
   an author's own on a bare canvas), and the runtime hands that painter exactly\
   the canvas child's layout box to draw into. A control with no intrinsic size\
   is therefore not merely small — the region is zero, `paint_progress` and\
   `paint_slider` return before their first stroke, and the control is INVISIBLE\
   while its painter runs. This sheet named none of the three until this block. */\
/* The sizes, and where each comes from. `canvas` is the one with a SPECIFIED\
   default: HTML gives a canvas intrinsic dimensions of 300 × 150 CSS pixels, so\
   that is used verbatim rather than reinvented. The other two are the native\
   controls' measured defaults, adapted only where this toolkit's painter differs\
   from a native one. A `<progress>` is ~160 × 16 and is taken as it stands — the\
   bar is a 6px stroke (`THICKNESS`) down the middle of the box, so 16 leaves 5px\
   of paper either side of it. An `<input type=range>` is ~129 × 14, and only the\
   shape of that transfers: this slider's thumb is a circle of radius 8\
   (`THUMB_RADIUS`), so a 14px-tall box would flatten it into a lens, and 20 is\
   the thumb plus 2px of air. Its width is the progress bar's 160 rather than 129\
   so the two bar-shaped controls share one rhythm.\
   Display is an APPROXIMATION: HTML makes both `canvas` and `progress`\
   inline-block, and this engine maps no inline-block (an unmapped display falls\
   to `block`), so all three are blocks and none of them shares a line with text.\
   That is the one thing here the engine's limits cost. */\
/* A canvas gets a box and nothing else — no border, no fill, no ink. Its inside\
   is drawn by a painter that is handed the WHOLE box and knows nothing about\
   padding or border: chrome from this sheet would be painted over by the first\
   thing the painter strokes, and any inset would move the pixels away from the\
   coordinates the painter measures against. `img` above is bare for the same\
   reason — replaced content decorates itself. */\
canvas {\
  display: block;\
  width: 300px;\
  height: 150px;\
}\
progress {\
  width: 160px;\
  height: 16px;\
}\
/* The slider rule is deliberately BARE — no padding, no border — and that is a\
   constraint rather than a taste. `SliderControl::fraction_at` turns a pointer x\
   into a value by insetting `THUMB_INSET` (8px) from the SLIDER ELEMENT's layout\
   box; `paint_slider` draws the thumb by insetting the same 8px from the CANVAS\
   CHILD's box. The two agree only while those are the same rectangle. Padding or\
   a border here insets the child and nothing else, so the thumb would stop\
   landing under the finger by exactly that much — worst at the two ends, where a\
   user aims most carefully, and with no other symptom. If this control ever\
   wants a frame, the frame and the fix ship together: both sides have to measure\
   one box. `the_pointer_track_and_the_painters_track_are_one_box`, in the\
   facade's `ua_stylesheet_vocabulary.rs`, fails the moment they part. */\
slider {\
  width: 160px;\
  height: 20px;\
}\
/* A slider takes focus — its semantic role is interactive, so it is in the Tab\
   ring — and moves under the arrow keys, so it wears the sheet's focus ring like\
   every other control. 3px of offset rather than the fields' 2, because the thumb\
   reaches the box edge and a closer ring would touch it. */\
slider:focus {\
  outline: 2px dashed #2b2a27;\
  outline-offset: 3px;\
}\
/* The canvas child of a drawn control, addressed as `*` because there is nothing\
   else to address it by: `Progress::contents` and `Slider::contents` build it\
   with no local name and no class (`Canvas::into_view_node` contributes no name\
   at all — the `img` defect this sheet's audit fixed, still open for the canvas\
   primitive). It is given the control's whole box, as a PERCENTAGE rather than as\
   the numbers above, and both reasons are about two boxes staying equal: a block\
   child already stretches to its parent's content width, but an auto height on a\
   childless box is 0, which is the zero-region case at the top of this block; and\
   an author who writes `slider { height: 40px }` moves the painter's box with the\
   control, where a repeated `20px` here would silently reintroduce the very\
   divergence the bare `slider` rule exists to avoid. (On hand-written `<progress>`\
   markup this reaches the author's own children too. That is the cost of the\
   widget's part carrying no name.)\
   LAST in the sheet on purpose: `canvas` above has the same specificity (0,0,1),\
   so source order is what keeps a control's canvas at the control's size —\
   including on the day someone gives the canvas primitive its missing local name\
   and that rule starts reaching these children. */\
progress > *, slider > * {\
  width: 100%;\
  height: 100%;\
}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sheet_is_a_self_contained_resource() {
        // What "package ресурсов" (§2.6) has to mean for a stylesheet that ships
        // as text: nothing in it reaches outside itself. There is no loader in
        // this crate and no base URL for one to resolve against, so an `@import`
        // or a `url(…)` would be a reference nobody could follow — it would parse,
        // then silently contribute nothing.
        assert!(
            !ua_css().contains("@import"),
            "the UA sheet imports nothing"
        );
        assert!(
            !ua_css().contains("url("),
            "the UA sheet references no external resource"
        );
    }

    #[test]
    fn the_palette_the_docs_name_is_the_palette_the_text_uses() {
        // The module doc states the Wireframe palette by hex value, and the
        // showcase/proof examples assert those exact numbers. A colour edited in
        // the text and not in the prose is a doc that lies about the look.
        for colour in [
            "#2b2a27", // ink
            "#faf9f6", // paper
            "#e4e2db", // hover fill
            "#f4f2ec", // field hover
            "#f2f0ea", // disabled fill
            "#b7b5ad", // disabled ink
            "#9b998f", // faint ink (the placeholder)
            "#d6d4cc", // hairline rule
        ] {
            assert!(ua_css().contains(colour), "{colour} left the sheet");
        }
    }

    #[test]
    fn a_drawn_controls_canvas_is_sized_after_the_bare_canvas_rule() {
        // The one property of this sheet that is carried by ORDER rather than by
        // any declaration in it. `canvas { width: 300px; height: 150px }` and
        // `progress > *, slider > *` have the same specificity — (0,0,1), one type
        // selector each, `*` counting for nothing — so whichever is written last
        // wins on a control's canvas child. Today the first rule cannot reach it at
        // all (`Canvas::into_view_node` contributes no local name), which is
        // exactly why the ordering is easy to break and impossible to notice:
        // reversing these two would do nothing whatsoever until someone fixes that
        // naming, and would then stretch every slider's painter box to 300 × 150
        // while the pointer kept measuring 160 × 20 — a thumb that no longer lands
        // under the finger, caused by a change in an unrelated file.
        let sheet = ua_css();
        let bare = sheet.find("canvas {").expect("the bare canvas rule");
        let child = sheet
            .find("progress > *, slider > * {")
            .expect("the drawn controls' canvas-child rule");
        assert!(
            bare < child,
            "the control's canvas child must be sized AFTER the bare canvas rule"
        );
    }

    #[test]
    fn the_slider_rule_carries_no_padding_and_no_border() {
        // The sheet's half of the slider's pointer/painter agreement, checked
        // where it is written. `SliderControl::fraction_at` insets 8px from the
        // slider ELEMENT's box and `paint_slider` insets 8px from the CANVAS
        // CHILD's box; padding or a border on this rule insets the child and
        // nothing else, and the thumb stops landing under the finger by that much.
        // The facade's `ua_stylesheet_vocabulary.rs` measures the two rectangles
        // against each other; this is the cheap version that names the cause.
        let sheet = ua_css();
        let start = sheet.find("slider {").expect("the slider rule");
        let rule = &sheet[start..start + sheet[start..].find('}').expect("its end")];
        assert!(
            !rule.contains("padding"),
            "no padding on the slider: {rule}"
        );
        assert!(!rule.contains("border"), "no border on the slider: {rule}");
    }
}
