Classifies this cause into its [`InvalidationKind`] depth.

The style-affecting causes ([`StyleChanged`](InvalidationReason::StyleChanged),
[`InlineStyleChanged`](InvalidationReason::InlineStyleChanged),
[`PseudoStateChanged`](InvalidationReason::PseudoStateChanged),
[`CustomPropertyChanged`](InvalidationReason::CustomPropertyChanged)) all map
to [`InvalidationKind::Style`]; the remaining causes map to their matching
depths.
