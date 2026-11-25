A bounded easing curve applied to normalized progress (spec §4.17).

These are the native compile targets of the common CSS timing functions:
`linear`, `ease-in`, `ease-out` and `ease-in-out`. Each maps normalized
progress `t ∈ [0, 1]` to eased progress in `[0, 1]` with `apply(0) == 0` and
`apply(1) == 1`. Full arbitrary `cubic-bezier(...)` / `steps(...)` timing
functions are an *adapter* concern: a compiler lowers them onto this enum (or
a keyframe sampling of the curve) rather than the domain modeling every
bezier directly.
