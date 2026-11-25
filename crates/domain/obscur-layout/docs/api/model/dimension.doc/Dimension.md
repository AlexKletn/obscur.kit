A sizing input expressed in layout's own vocabulary (spec §4.1).

A [`Dimension`] describes how a length is specified before resolution:
automatic (content/context driven), an absolute number of logical pixels,
a percentage of some base length, or one of the **intrinsic** sizing
keywords, which are answered by measuring content rather than by arithmetic.
All values are fractional logical pixels; no rounding is applied here
(spec §4.6).
