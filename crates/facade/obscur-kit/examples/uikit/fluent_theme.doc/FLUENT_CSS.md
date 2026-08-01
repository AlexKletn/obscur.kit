The complete **Fluent / Acrylic** stylesheet (theme index 7) — the eighth and
final UI-Kit theme, and the one that exercises the freshly-shipped
`backdrop-filter: blur()` (frosted-glass / acrylic material). Windows-11
Fluent: Segoe UI face (approximated by the host font), a single vivid blue
accent (`#0F6CBD`), a red error (`#C50F1F`), soft 6–12px rounding, thin light
1px borders, and translucent surfaces layered over a light "mica" page wash.

# Acrylic — how the frost is made to READ

`backdrop-filter: blur()` only shows if there is *variation* behind the panel
(blurring a flat fill returns the same fill). So the page (`#app`) is painted
with a **varied light gradient** — a gentle diagonal blue→neutral→violet→teal
mica wash over `#eef1f6` — and the cards/menus/dialog are **acrylic**:
a translucent `rgba(255,255,255,0.55)` fill + `backdrop-filter: blur(24px)` +
a hairline light border. The gradient shows through each card *frosted and
tinted*, which is the acrylic. This is GPU-only: the Vello backend frosts the
backdrop; the CPU backend honestly skips the blur (the fill is still
translucent, so the tint still bleeds — just un-blurred).

Layout numbers (row heights, card widths, control sizes) are shared with the
other themes so the fixed [`view`](super::view) fits. Fluent-specific approximations:
- **Segoe UI / JetBrains Mono** → the host CPU font (shapes, not the face).
- **`letter-spacing` / `text-transform`** (eyebrow tracking, uppercasing) are
  not in our property set → dropped; eyebrow labels are already uppercased in
  the view. **`backdrop-filter: saturate()`** (spec pairs it with blur) is not
  modelled → silently dropped; only the `blur()` is honored.
- The spec's absolutely-positioned blurred colour *blobs* can't be added to the
  fixed view → their vivid tint is folded into the `#app` mica gradient instead.
