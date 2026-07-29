# 07 — Text and fonts

Text is the part of the kit where "it compiles" and "it draws" are furthest
apart, so this page is mostly about wiring.

A runtime with nothing installed **lays text out and paints no glyphs**: the
default shaper is a deterministic mock with plausible metrics, and the default
font resolver is empty. That is a deliberate default — layout is testable
without a font on the machine — but it means an application that skips the text
stack gets correct boxes and blank text.

---

## Installing the stack

```rust
use obscur_kit::bootstrap::KitBootstrap;

let boot = KitBootstrap::new();
let fonts = boot.system_font_set().unwrap_or_default();
let installed = boot.install_text_stack(&mut runtime, &fonts);
for (handle, bytes) in installed.iter() {
    renderer.register_font(handle, bytes.to_vec());
}
```

`App` does exactly this. Four ports are replaced at once: the shaper (HarfRust),
the itemizer (Unicode), the font resolver, and the face metrics the shaper sizes
line boxes from.

| `KitBootstrap` | |
|---|---|
| `system_font_set() -> Option<FontSet>` | the host's `sans-serif`, `serif` and `monospace` through fontconfig |
| `install_text_stack(&mut UiRuntime, &FontSet) -> InstalledFonts` | installs everything and returns the handle→bytes table |
| `font_registry() -> FontRegistry` | the registry alone |
| `text_shaper_from_bytes(bytes) -> Box<dyn TextShaper>` | a shaper over one face |
| `text_shaper_without_font(FaceMetrics) -> Box<dyn TextShaper>` | a shaper that answers metric queries from a registry |
| `text_itemizer() -> Box<dyn TextItemizer>` | |

## `FontSet` and `InstalledFonts`

```rust
FontSet::new()
    .with(FontFamily::generic(GenericFamily::SansSerif), FontAttributes::default(), bytes)
```

| `FontSet` | `new`, `push(FontFaceBytes)`, `with_face`, `with(family, attributes, bytes)`, `faces`, `len`, `is_empty` |
|---|---|
| `FontFaceBytes` | `new(family, attributes, bytes)`, `bytes()` |
| `InstalledFonts` | `iter() -> impl Iterator<Item = (u32, &Arc<[u8]>)>`, `handle(index)`, `take_discovered() -> Vec<(u32, Arc<[u8]>)>`, `len`, `is_empty` |

`install_text_stack` appends the **bundled last-resort face** to whatever you
pass, so a set that covers nothing still draws Latin.

Vendored under `assets/fonts/`: `NotoSans-Regular.ttf` and
`NotoSansMono-Regular.ttf`, with licence and checksums beside them.

## Fallback arrives mid-frame

The bundled face carries 2,468 codepoints. Anything outside the application's
chain — a CJK filename, an emoji, `⌫` on a key — resolves to no face, and **an
unresolved run paints no glyphs at all**. Not a box: nothing. A blank where a
control should be reads as a broken control, not as a missing font.

So the kit asks the host, per character, for a covering face:

```rust
// KitBootstrap wires this when the `font-fontconfig` feature is on
FontconfigFallback::new()   // implements SystemFontFallback
```

The face is discovered *during* the frame that needed it, which is why the
hand-off is a drain rather than a one-time list:

```rust
for (handle, bytes) in installed.take_discovered() {
    renderer.register_font(handle, bytes.to_vec());   // before you present
}
```

Miss this and the run resolves to a handle the renderer does not have — and a
handle it does not have falls back to handle `0`, so you get *the wrong face's
glyph ids*, which is visually worse than nothing.

### What fallback does not do

- **Per run, not per character.** Text is itemized by script, a face is chosen
  per item, so one uncovered character pulls its whole item into the host's
  face: `ab⌫cd` comes out entirely in the fallback face. Splitting an item at
  coverage boundaries would be the fix and is not built.
- **`.ttf` / `.otf` only.** A host whose CJK font is a `.ttc` collection still
  draws nothing.
- **Coverage, not style.** A fallback face is taken at its default weight and
  slant.

## Platform support

| | |
|---|---|
| Linux | **wired** — fontconfig enumeration and per-character `fc-match` |
| Web | **partial** — a host provides faces; `queryLocalFonts` is the intended path |
| macOS, Windows | **unwired** — enumeration is not implemented |

The `fc-match :charset=<hex>` answer is treated as a *suggestion*: the registry
re-verifies the character against the face's own cmap before trusting it, because
a non-covering face would become a permanent wrong answer.

## The text pipeline

`obscur_kit::text` holds the domain tier: `TextStyle` and spans, itemization,
shaping (`TextShaper`), paragraph layout and line breaking, hyphenation
(`Hyphenator`), IME composition types, and the font model (`FontFamily`,
`FontAttributes`, `GenericFamily`, `FontId`, `FallbackFace`, `SystemFontFallback`).

Line-breaking, `white-space`, `overflow-wrap`, `word-break`, `text-overflow`,
`letter-spacing`, `word-spacing`, `tab-size` and `text-transform` are all real
CSS-driven behaviour, not approximations.

## Locale

```rust
rt.set_locale(&LanguageTag::parse("ru-RU")?);
```

Defaults to `und`. The one wired production consumer is the shaper's language
hint. `obscur_kit::localization` also has CLDR plural categories and a message
catalog.

> **partial** — dates, numbers and currencies are not formatted, and the RTL
> frame path is open. `direction: rtl` computes and reaches layout; a full
> bidi frame is not there.

## Debugging blank text

In order of likelihood:

1. **No text stack installed** — the mock shaper measures and draws nothing.
2. **The renderer never got the face** — `installed.iter()` not forwarded, or
   `take_discovered()` not drained before present.
3. **The character is outside every registered face** and the host has no
   fontconfig, or has it and covers nothing.
4. **The box is there and empty** — check `UiRuntime::layout_boxes()`; a
   zero-width content box clips the run away, which is a styling problem, not a
   font one.
