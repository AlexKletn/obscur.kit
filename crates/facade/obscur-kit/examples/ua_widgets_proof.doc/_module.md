**Proof: the UA default stylesheet draws the wireframe indicator sub-parts.**

Lays out BARE built-in form controls (no author CSS for the controls — only a
neutral `.page` layout harness) with the toolkit's UA default stylesheet
installed at the lowest cascade origin, and rasterizes them through the
software [`CpuRenderer`] into `target/ua_widgets_proof.png`:

- **Checkbox** — unchecked (empty 20×20 box), checked (filled box + ✓),
  indeterminate (filled box + thin bar), and disabled (dashed muted box).
- **Radio** — unselected (empty circle), selected (circle + filled dot), and
  disabled (dashed muted circle).
- **Select** — closed (▼) and open (▲).

Every indicator comes entirely from the UA sheet keying on the sub-part
classes (`.checkbox-box` / `.radio-dot` / `.select-arrow`) the widgets emit.

Run it:
`cargo run -p obscur-kit --example ua_widgets_proof \
  --features "render-cpu text-harfrust text-unicode"`
