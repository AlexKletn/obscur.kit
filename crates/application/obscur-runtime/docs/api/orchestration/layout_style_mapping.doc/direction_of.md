Resolves the computed `direction` keyword to a [`Direction`], defaulting to
[`Direction::Ltr`] (the CSS initial).

Compares the interned symbol against the reserved
[`layout_keyword::DIRECTION_RTL`] handle; an absent or unrecognized value maps
to [`Direction::Ltr`]. This is the same `direction` the style crate already
resolved to fold logical box-model properties onto physical ones; the layout
bridge re-reads it because flow layout also needs the geometric half (which
physical edge an over-constrained child settles against).
