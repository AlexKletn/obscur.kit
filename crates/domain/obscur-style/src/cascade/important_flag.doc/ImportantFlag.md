Whether a declaration is `!important` (§4.14).

Within the same origin band, an `!important` declaration outranks a normal
one, so the type derives [`Ord`] with [`ImportantFlag::Normal`] <
[`ImportantFlag::Important`] to match. The full interleaving of importance,
origin and the runtime-override layer is computed by
[`crate::cascade::CascadePriority`].
