The same identity again, as an `i64`.

A loop's `key` is a [`ViewKey`](obscur_kit::view::view::ViewKey), which converts
from `u64`, `&str` and `String`; a prop is a `ViewValue`, which carries `i64`
and not `u64`. The two vocabularies do not share a numeric type, so an
application that wants one number in both carries it twice — flattened into the
model here, because markup cannot call a conversion.
