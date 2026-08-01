Replays `commands` onto `scene` under a `base` transform, in paint order.

When `extract` is `Some`, a `PushLayer` carrying a
[`Blur`](obscur_scene::layer::LayerEffect::Blur) effect (and a clip whose
bounds give the blur region) is peeled off: its bracketed commands become a
separate sub-scene pushed into `extract`, and are **not** drawn here. When
`extract` is `None` (a sub-scene render), such a layer is drawn as an
ordinary layer (its content painted un-blurred). Pops are balanced against a
depth counter, so a malformed list never panics: a stray pop is ignored and
open layers are closed at the end.
