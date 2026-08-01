Interpolates between two straight-alpha [`Color`]s in **premultiplied** space at
`t ∈ [0, 1]`, returning a straight-alpha result (css-color-4 §12.3, obscur.kit
§5.7).

Each endpoint is premultiplied (`rgb·a`), the four premultiplied channels are
lerped, and the result is un-premultiplied by the interpolated alpha. This is
what keeps a `transparent → blue` stop from fading through grey: at the midpoint
the colour stays full-strength blue at half alpha, not half-strength blue. A
straight component-wise lerp (see [`lerp_color`]) is the classic wrong answer,
silently visible only where the endpoints differ in alpha.
