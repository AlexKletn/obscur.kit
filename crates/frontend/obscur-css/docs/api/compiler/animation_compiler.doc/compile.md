Extracts every [`AnimationRef`] declared across `decls`, interning each
animation name in `interner`.

`animation` shorthand layers each contribute one ref (name + that layer's
duration, delay, iteration count, direction and fill mode); bare
`animation-name` idents each contribute one ref carrying the
`animation-duration` longhand's value (or `0` when absent) and the
playback longhands. An `animation-name: none` layer is skipped.

A playback longhand declared **after** the `animation` shorthand
overrides it on every shorthand-derived ref (`animation: a 1s;
animation-direction: alternate`); one declared *before* it is reset by
the shorthand, as the cascade requires. Per-layer longhand lists are not
modelled: one longhand value applies to all of a rule's refs.
