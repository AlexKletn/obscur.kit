Rounds a non-negative iteration count to whole iterations, clamped into
`u32`. A count above `u32::MAX` saturates (it is indistinguishable from
`infinite` in practice, but is still finite).
