Reads a node's computed `backdrop-filter` blur radius (logical px), or
[`None`] when no positive-radius `blur()` is authored (the byte-identical
no-effect path).

`backdrop-filter` reuses the [`PropertyValue::Filter`](obscur_style::property::PropertyValue::Filter)
op list (only `blur()` ops survive its lowering). A value may list several
`blur()` ops; the **maximum** authored radius is used (multiple backdrop blurs
on one element are uncommon; documented). The coordinator emits this radius as a
[`BackdropFilter`](obscur_scene::command::PaintCommand::BackdropFilter) command
frosting the backdrop under the node's border box.
