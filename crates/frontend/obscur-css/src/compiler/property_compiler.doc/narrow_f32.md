Narrows a parser `f64` into the `f32` the style/motion domains use.

The tokenizer parses into `f64` for range while the presentation domains
store `f32`; this single checked-narrowing point keeps the (intentional)
truncation contained.
