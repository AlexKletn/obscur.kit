**Interactive windowed imported-UI-Kit theme switcher → Vello (GPU).**

The live sibling of `uikit_png.rs` / `uikit_vello_png.rs`: it renders the *very
same* shared [`uikit`] view (the top theme-switcher bar over the component
gallery) through winit 0.30 + Vello/wgpu into a genuine on-screen window — and
makes the switcher **clickable**. Clicking a theme tab swaps that theme's
stylesheet and rebuilds the identical view *live*, so the whole gallery re-skins
in place at full GPU fidelity.

Because this is the GPU backend, the gradient/shadow-heavy themes render for
real, not the CPU approximation:

- the **Skeuomorphic** tab (index 4) paints its `linear-gradient(#light,#dark)`
  faces as real smooth ramps and its `inset`/outset `box-shadow` gloss + cast
  shadows as real Gaussian blurs (the CPU PNG shows a flat first-stop fill and a
  flat offset rect);
- the **Material** tab (index 1, the default) paints its tonal fills, elevation
  shadows, and the `mini-cover` gradient at full fidelity;
- the remaining six tabs are the neutral placeholder stylesheet for now — a
  click still swaps to them live (they just look plain, by design).

The view structure is *identical* across every tab (that is the whole point of
the one-view/many-stylesheets architecture); a tab click is therefore a **pure**
stylesheet swap plus a rebuild of the same `uikit::view(i)` with tab `i`
highlighted.

# Device scale, wheel scroll & zoom

The window is **device-scale aware**: the render scale is the display's HiDPI
`scale_factor` — **not tied to the window size**. It hands the runtime its
swapchain via [`UiRuntime::set_surface`], which points the viewport at
`physical_extent / effective_scale` and sets the device scale to that same
factor — the runtime then brackets the whole scene in a root `scale(s, s)`
transform, so vectors, text, and gradients rasterize crisply at their authored
CSS size. Because the scale is fixed to the display density, **resizing the
window reflows the layout** (the gallery divides the real width; the UI is *not*
zoomed up and down with the window) — exactly like a real application. Any
content taller than the window is reached by **scrolling**: the mouse wheel
drives a vertical `scroll_y`, pushed to the runtime as
[`UiRuntime::set_content_offset`] (a root `translate` composed with the device
scale). A user **zoom** multiplies on top: `Ctrl+=` / `Ctrl++` zoom in (cap
500%), `Ctrl+-` zooms out (min 30%), `Ctrl+0` resets zoom **and** scroll. The
device scale + scroll clamp are recomputed on resize / scale-factor change /
zoom. Pointer coordinates map through the device scale + scroll offset
automatically (the runtime divides by the scale and adds the offset back), so
tab hit-testing is correct at any scale, zoom, or scroll position.

Run it (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example uikit_window --features full`

Click the **Material** vs **Skeuomorphic** tabs to swap the look live;
**mouse wheel** scrolls vertically; `Ctrl` +/-/0 to zoom / reset.

# Environment note

Like `showcase_window.rs`, this needs a display and a usable GPU adapter; in a
headless environment it compiles but exits cleanly on `resumed` when no
adapter/surface can be acquired (it never panics on that path).
