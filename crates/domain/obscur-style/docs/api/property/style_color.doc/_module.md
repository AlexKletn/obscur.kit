CSS color as modelled in style (not the renderer's paint color).

[`StyleColor`] is the *authored* color the cascade works with: sRGB with
straight (non-premultiplied) alpha, each channel in `0.0..=1.0`. It is a
pure style concept and deliberately has **no dependency on the scene**.

Conversion to the renderer's paint color — which is premultiplied and in a
linear working space — happens later, in a dedicated scene converter. Keeping
that conversion out of this type preserves the authored values exactly as the
stylesheet expressed them.
