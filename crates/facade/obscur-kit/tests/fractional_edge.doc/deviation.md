How far the rasterized extent strays from a true circle of radius
`56 * 1.6 / 2`, as (overshoot, undershoot) in pixels over every row: how far
paint reached past where that circle can reach in the row, and how much of where
it certainly covers went unpainted.

A flattened side is undershoot. It is returned rather than asserted so a control
case can check that a genuinely clipped render does register here.
