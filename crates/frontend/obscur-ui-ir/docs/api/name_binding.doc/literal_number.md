The number a literal names, if it names one.

`Text` is parsed because that is what a quoted attribute lowers to; a
non-finite result is refused, since `"inf"` and `"nan"` parse but no scale can
use them.
