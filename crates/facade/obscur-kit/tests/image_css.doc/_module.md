End-to-end verification that **authored `background-image: url(...)` CSS**
reaches paint through the *real* live pipeline: a real PNG on disk is loaded by
a sandbox-rooted `FsResourceLoader`, decoded by the `image`-crate `ImageDecoder`,
cached as a `SceneImage`, and painted as a `DrawImage` over the element's box
(spec §5.15 load, §5.16 decode, §5.2 paint) — closing audit finding V6 for
images ("loaded bytes never become a usable image").

The test authors CSS text, compiles it through the real `obscur-css` frontend,
renders one frame on a [`UiRuntime`] wired with the facade's real filesystem
loader + image decoder (via [`KitBootstrap::install_image_resources`]), and
asserts both that the published [`DisplayList`] carries a
[`PaintCommand::DrawImage`] and that the CPU (tiny-skia) backend paints the
image's pixels inside the element box (not the transparent background).
