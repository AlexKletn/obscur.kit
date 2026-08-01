The styling.

Two rules it follows, both learned from what the kit actually does rather
than from what CSS says:

- **No `width: 100%` on anything with padding.** A percentage width inside a
  percentage-width padded parent resolves against the parent's *border* box
  here, so each nesting level grows by its parent's padding and the content
  spills out of the window. Flex stretches a child across the cross axis
  anyway, so stating no width at all is both shorter and exact.
- **Type is set with `clamp(min, vmin, max)`**, not in fixed pixels. Keys
  scale with the window and a fixed label does not, so it drifts visibly out
  of proportion — the label stays the size it was while its key grows around
  it. `min(<vmin>, <px>)` rather than `clamp`: a **floor** is what makes the
  text stop shrinking while the key carries on, so at a squeezed window the
  label ends up several times wider than the key it sits in. A ceiling alone
  keeps type sane on a wall-sized window and lets it vanish gracefully on a
  squeezed one; `overflow: hidden` on the key catches whatever is left.
- **No fixed heights on the rows inside a panel.** The panels are `flex`-sized
  and shrink with the window; a row pinned to `52px` inside one does not, so
  past a certain size it is pushed out of the panel that is supposed to
  contain it — which reads as content drifting out of place. The rows size to
  their (already scaling) text instead. `.tape-row` is the one exception and
  must stay fixed: it is a virtual-list row, and the runtime sizes its window
  and spacers from exactly that number.
- **A styled `button` resets the UA `padding` and `border`.** Every key here
  is a real [`Button`], so the kit's default sheet gives it `padding: 11px
  20px` — 40px of horizontal padding, which a 30px-wide key does not have.
  The content box collapses, `overflow: hidden` clips what is left, and the
  key renders as an empty pill. That is the browser contract (a `<button>`
  carries UA padding until an author says otherwise) and it bites exactly
  where a demo is least likely to look: on a small window, at a display scale
  of 2, where the *logical* viewport is half the pixels on screen.
- **Corner radii scale like type does.** `border-radius: 14px` on a 30px key
  is a pill, not a rounded key. `min(3vmin, 14px)` keeps the intended shape at
  every size.

**Transparency is the root's alpha and nothing else.** The window is always
created transparent with a premultiplied composite mode, so the bottom-most
fill decides what the compositor sees: `.safe` at `0.62` lets a blurring
compositor read the wallpaper through the whole app. There is deliberately no
API for it — a flag would be a second source of truth about the same pixel.
The panels and keys are translucent too, a little less so as they go up, so
the blur stays visible through them while the text stays fully opaque and
legible.

`env(safe-area-inset-*)` on `.safe` is this app's whole awareness of the
device it is on: a notched phone pads itself, a desktop window does not.

The body is capped and centred rather than stretched. A tiling compositor
hands this window whatever it likes — the owner's first run of it got roughly
1200×1500 — and a keypad that grew to fill that has 440px keys, which reads as
a bug rather than as a design. Capped, it is a calculator sitting in the
middle of the screen; on a phone-sized window the cap is never reached and it
fills as before.
