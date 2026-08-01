The alpha multiplier the software backend applies to a `box-shadow` fill to
fake the softness a real gaussian blur would give (this backend has no blur).
A documented approximation (spec §5.2): the offset rounded rect is filled at
half the shadow color's alpha rather than blurred.
