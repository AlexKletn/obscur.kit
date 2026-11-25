One `filter` layer — a `blur(radius)` and/or a color-matrix transform —
extracted from the display list as a separate sub-scene the renderer
post-processes and composites (spec §5.2).

The scene coordinator brackets a filtered element's subtree with a
`PushLayer(filter)` … `PopLayer`; [`SceneConverter::convert_with_blurs`] detects
that bracket, converts the bracketed commands into their **own** [`Scene`]
(omitting them from the main scene so they are not also drawn unfiltered), and
records this descriptor. The renderer then renders [`scene`](Self::scene) into
an offscreen texture, applies the owned color-matrix pass (if
[`color_matrix`](Self::color_matrix) is set) and/or a separable gaussian pass
(if [`radius`](Self::radius) `> 0`), and composites the result back onto the
target at [`bounds`](Self::bounds).

The sub-scene draws in the **same absolute coordinates** as the main scene (any
enclosing transform is already baked in), so the renderer offsets it to a
margin-padded offscreen texture and composites it back at its absolute position.

Despite the historical name, a layer may carry a color matrix with **no** blur
(`radius == 0`) — the color-only filter path.
