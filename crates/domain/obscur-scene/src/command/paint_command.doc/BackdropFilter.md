Blur the **backdrop** — everything already painted behind this point —
within `bounds`, clipped to the rounded `radii`, in place (CSS
`backdrop-filter: blur(<len>)`, the frosted-glass / acrylic effect, spec
§5.2). A LEAF command (no nesting effect).

Unlike [`PaintCommand::DrawShadow`] or a `filter: blur` layer (which blur
the element's OWN content), this command blurs the accumulated frame as
painted so far — the content BEHIND the element — so it is inherently
**order-dependent**: a renderer must have already composited every prior
command before applying it. The coordinator emits it at the element's paint
position, **before** the element's own background / border / content, so the
element's (usually translucent) background then draws OVER the frosted
backdrop → acrylic.

`bounds` is the element's border box and `blur` the CSS blur *radius* in
logical pixels (`0` = no effect). Only the rounded-`radii` area is frosted;
pixels outside the rounded shape are left sharp. Backends without a real
backdrop pass (e.g. the CPU rasterizer) may skip it (honest no-op).
