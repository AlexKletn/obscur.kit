**Proof: the built-in UA default stylesheet gives bare controls a base look,
and an author sheet overrides it.**

Renders three BARE built-in controls — a [`Button`], an [`Input`], and a
[`Checkbox`] — twice through the software [`CpuRenderer`], stitching the two
frames into one `target/ua_default_proof.png`:

- **Top row — UA default only.** The runtime has the toolkit's UA default
  stylesheet installed at the lowest cascade origin
  ([`CascadeOrigin::SystemDefault`]) via [`ua_stylesheet`], and NO author CSS
  for the controls (only a neutral layout harness that arranges them). The
  controls show their neutral base look: border, padding, radius, background.
- **Bottom row — same, plus a one-line author override.** An author sheet at
  `GlobalApp` adds `button { background: #cc0000 }`. Because the UA sheet sits
  below every author sheet, the button turns red while the untouched input and
  checkbox keep the UA look — proving the UA sheet is a true, overridable base.

It leans only on properties the CPU renderer paints faithfully (border,
padding, radius, background, color, font-size).

Run it:
`cargo run -p obscur-kit --example ua_default_proof \
  --features "render-cpu text-harfrust text-unicode"`
