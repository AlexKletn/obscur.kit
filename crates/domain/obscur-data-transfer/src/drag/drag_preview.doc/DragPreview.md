The placement descriptor for a drag's visual image (spec §6.21 previews).

While a drag is in flight the toolkit shows a preview that follows the
cursor. This value is only the *placement* description: its [`size`] in
logical pixels, the [`cursor_offset`] hotspot (where within the preview the
pointer sits, so the preview tracks the cursor correctly), and its
[`opacity`]. The actual pixels are produced by the scene; this descriptor
carries no image data.

[`size`]: DragPreview::size
[`cursor_offset`]: DragPreview::cursor_offset
[`opacity`]: DragPreview::opacity
