Composes a paint-ordered [`DisplayList`] like [`compose`](Self::compose),
additionally painting `background-image: url(...)` images that have already
been resolved into the `images` lookup.

`images` maps a computed `background-image` URL (the raw authored string) to
its decoded [`SceneImage`]. The runtime resolves URLs to images *outside*
this pure coordinator (through a resource loader/decoder) and hands the
ready map in, so composition stays deterministic and IO-free: a node whose
`background-image` is a URL present in `images` emits a
[`DrawImage`](obscur_scene::command::PaintCommand::DrawImage) stretched over
its box (painted OVER `background-color`, UNDER its border and content); a
URL absent from the map (not yet loaded, or failed) paints nothing — exactly
the [`compose`](Self::compose) behaviour. An empty map reproduces
[`compose`](Self::compose) byte-for-byte.
