Resolves a relative `font-weight` (`bolder` / `lighter`) against the parent's
computed weight, per the CSS fixed table (§4.14).

The parent weight is rounded to the nearest hundred and clamped to the
`100..=900` buckets; each bucket maps to a fixed `(bolder, lighter)` pair:
100/200/300 → (400, 100), 400/500 → (700, 100), 600/700 → (900, 400),
800/900 → (900, 700).
