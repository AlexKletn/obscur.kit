Interpolates two straight-alpha sRGB [`Color`]s in **Oklab with premultiplied
alpha** at parameter `t ∈ [0, 1]`, returning a peniko sRGB color.

CSS Color 4 §12.2 interpolates in the (rectangular) Oklab space over
premultiplied components. Each endpoint is taken from gamma sRGB → linear sRGB
([`srgb_decode`]) → Oklab ([`linear_srgb_to_oklab`]), premultiplied by its
alpha; the four components `L,a,b,α` lerp linearly; the result is
un-premultiplied and mapped back with [`oklab_to_srgb`]. A zero interpolated
alpha yields a fully transparent black (no division by zero).
