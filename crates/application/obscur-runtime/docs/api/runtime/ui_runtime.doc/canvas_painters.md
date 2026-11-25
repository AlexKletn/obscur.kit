The painters registered on this runtime, keyed by
[`PainterKey`](crate::runtime::PainterKey).

Not a `<canvas>` facility: since `-obscur-painter`, **any** element can carry a
painter, and the key says how it got one — a `Symbol` the node bound itself
(what a shipped `Slider` or `Progress` does) or a `Name` a stylesheet gave it.
