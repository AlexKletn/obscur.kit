The **interactive Wireframe-Kit** scene — authored once, driven by both the
live windowed Vello example (`wireframe_live_window.rs`) and the headless
CPU→PNG proof (`wireframe_live_headless.rs`).

# The point

Every control here is **BARE**: a real [`Button`] / [`Input`] / [`Checkbox`] /
[`Radio`] / [`Select`] (or a raw `<a>`), carrying **no** author CSS for its
appearance. All of the control look — the paper fill, the 1.5px ink border, the
2px radius, the hover fill shift, the dashed focus ring, the inverted `:active`,
the checkbox `✓` box, the radio dot, the select `▼`/`▲` — comes entirely from
the kit's **UA default stylesheet** ([`obscur_kit::ua_stylesheet`]), which the
two drivers install on the runtime. This demo therefore *proves* the UA look:
remove the installed default sheet and every control goes unstyled.

The scene authors only the **LAYOUT CHROME** — the graph-paper page, the
centered column, the masthead card (title + mono subtitle + description + the
five state-legend chips), the numbered section cards, the captions, and the
footer — via `.wf-*` classes. It never writes a `button { … }` / `input { … }` /
`checkbox { … }` / `.checkbox-box { … }` / `a { … }` rule, so the UA default
shows through unmodified.

Like the other shared scenes it is **backend-pure**: it names only the toolkit's
view / widget / primitive / CSS authoring surfaces (`obscur_kit::*`) and never a
renderer, window, or GPU type, so the same [`view`] + [`stylesheet`] feed a
software rasterizer and a GPU surface unchanged.

# Sections (all LIVE bare controls)

1. **Button** `<button>` — a live [`Button`] (`:hover` fill shift, `:active`
   invert, `:focus` ring; a click bumps the status counter) beside a disabled one.
2. **Text Field** `<input>` — a live [`Input`] (click → focus ring + caret,
   type → text) beside a **read-only** one (`.readonly(true)`, HTML `readonly`:
   focusable + selectable but not editable).
3. **Textarea** `<textarea>` — a live **multi-line** [`Input`]
   (`.multiline(true)`), which emits the reserved CSS type `textarea`. The UA
   sheet makes a bare `input` **single-line** (`white-space: nowrap`) but a
   `textarea` **multi-line** (`white-space: pre-wrap`), so this field WRAPS at
   its author `.wf-area` `360×96` box AND preserves newlines — a real textarea:
   Enter inserts `\n`, ArrowUp/ArrowDown move between lines, all on the UA base.
4. **Select** `<select>` — a live [`Select`] state machine. Closed, the header
   shows the current label + ▼. Open, the widget emits a real **dropdown popup**
   — a `.select-list` of stacked `.select-option` rows positioned directly BELOW
   the header (arrow ▲), the selected row inverted. A click while open picks the
   next option and closes. Per-**option** hit-routing is NOT a bare-`Select`
   capability (the widget bubbles one event for the whole control), so "pick" is
   a cycle-to-next; noted. Beside it a disabled one.
5. **Checkbox** `<input type=checkbox>` — a live [`Checkbox`] (box `✓` on click),
   a tri-state one, and a disabled one.
6. **Radio** `<input type=radio>` — a live group of three [`Radio`]s sharing one
   selection (the chosen one shows the filled dot), plus a disabled one.
7. **Text Link** `<a>` — a bare `<a>` element showing the UA underline and the
   `:hover` bold.

The purely-static spec-only groups from the source kit (range / progress / meter
/ file / color / fieldset / table / scrollbar) are **skipped** — this demo
prioritises the genuinely interactive controls.
