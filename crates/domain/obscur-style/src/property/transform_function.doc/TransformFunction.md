One function of a `transform` list, as authored — kept in author order, and kept
UNRESOLVED.

`transform: translate(2em, 10vw)` cannot be a matrix at CSS-lower time: `em` is
the element's font size and `vw` is the viewport, and the compiler knows
neither. The old lowering answered that by dropping the whole declaration, and
answered `translate(50%)` by substituting `0` — a lie that painted as "the CSS
didn't take". This type is the alternative: the function survives to
computed-value time carrying its [`LengthExpr`]s, and
[`absolutize_value`](crate::computed) folds it against a
[`LengthContext`](crate::property::LengthContext) exactly as it folds a
`box-shadow` offset.

- [`TransformFunction::Matrix`] — every function whose value is already a number
  at parse time (`scale`, `rotate`, `skew`, `matrix`, and a `translate` written
  in absolute units), pre-multiplied into its 2D affine `[a, b, c, d, e, f]`.
- [`TransformFunction::Translate`] — a `translate`/`translateX`/`translateY`
  whose distance still depends on the font, the viewport or the box.

A `%` distance resolves against the ELEMENT'S OWN border box, which is a used
value — no length context can carry it, so [`resolve`](TransformFunction::resolve)
answers [`None`] rather than `0`. See `docs/api/12-reality.md` for what is owed
before a percentage translate can paint.
