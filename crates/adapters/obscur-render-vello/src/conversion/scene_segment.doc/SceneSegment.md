One segment of a display list split at [`BackdropFilter`] boundaries by
[`SceneConverter::convert_segments`](super::SceneConverter::convert_segments)
(spec §5.2, CSS `backdrop-filter`).

A `backdrop-filter` frosts the frame accumulated so far, so the renderer must
composite content and frost the backdrop **in order**. The converter emits an
alternating sequence: content to render, a backdrop to frost, more content, …

[`BackdropFilter`]: obscur_scene::command::PaintCommand::BackdropFilter
