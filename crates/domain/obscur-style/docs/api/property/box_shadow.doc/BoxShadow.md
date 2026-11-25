A single CSS `box-shadow` layer, as authored in a stylesheet.

A shadow is two required offsets (`offset_x` / `offset_y`) plus an optional
`blur` radius, `spread` distance, `color`, and an `inset` flag. The lengths
are kept symbolic ([`Length`]) so relative units resolve at the layout/compute
step, exactly like every other length-typed property; the scene coordinator
turns them plus the painted box's border box into the concrete
`obscur_scene::command::PaintCommand::DrawShadow` a renderer consumes.

`color` is [`None`] when the author omitted it: CSS resolves an unset shadow
color to `currentColor`, which has no computed value, so the coordinator
substitutes the element's computed `color` at paint time (mirroring the
`border-*-color` fallback).

Because [`Length`] and [`StyleColor`] carry `f32` data, this type derives
[`PartialEq`] but not [`Eq`]/[`Hash`].
