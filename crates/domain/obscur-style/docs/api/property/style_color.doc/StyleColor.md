An sRGB color with straight alpha, as authored in a stylesheet.

Each channel (`r`, `g`, `b`, `a`) is nominally in the range `0.0..=1.0`.
Values are stored as-authored and are **not** premultiplied; alpha is kept
separate from the color channels. This is the style-side representation only;
see the module docs for the conversion to renderer paint color.

Because the channels are `f32`, this type derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
