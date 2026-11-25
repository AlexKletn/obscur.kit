Blends toward `other` component-wise, returning `None` on a variant
mismatch (spec §4.17).

`t` follows the [`Interpolatable`] contract: normalized progress in
`[0, 1]`, not clamped. When both operands hold the same variant the
inner value is interpolated and wrapped back up in `Some`; when the
variants differ the two endpoints are type-incompatible and `None` is
returned rather than a nonsensical blend.
