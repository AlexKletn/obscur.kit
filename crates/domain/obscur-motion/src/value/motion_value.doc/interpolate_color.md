Interpolates two straight-alpha sRGB colors with premultiplied alpha
(css-color-4 §13.3): premultiply rgb by alpha, lerp all four components, then
un-premultiply by dividing rgb by the interpolated alpha. When the
interpolated alpha is zero the un-premultiplied rgb is the premultiplied value
(zero), matching the spec's zero-alpha rule.
