Binds an integer count.

# The digits are ASCII, and that is a gap, not a decision

`literal` is `count`'s plain Rust formatting: `-1234` renders `-1234`,
never `−1 234` and never `١٢٣٤`. Locale-aware number formatting — digit
system, grouping separator, minus sign — is **not built** (see the crate
docs). The *category* is fully locale-aware; the digits are not.
