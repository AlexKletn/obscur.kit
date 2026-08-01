Wraps like [`wrap`](Self::wrap) but sizes each line box from the mixed-font
line-box **strut** (CSS 2.1 §10.8.1): the `strut` (the element's own
primary-font metrics + used `line-height`) plus each segment font's metrics
from `seg_metrics`. See
[`wrap_strut_hyphenated`](Self::wrap_strut_hyphenated) for the formula.
