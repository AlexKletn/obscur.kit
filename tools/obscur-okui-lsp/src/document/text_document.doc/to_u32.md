Converts a `usize` byte length/offset to `u32`, saturating at
[`u32::MAX`]. Documents larger than 4 GiB are out of scope for this core.
