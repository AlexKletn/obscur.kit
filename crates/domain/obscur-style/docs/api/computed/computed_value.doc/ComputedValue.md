A fully-resolved, concrete computed value for a single property.

The wrapped [`PropertyValue`] is invariantly concrete: it is never one of the
CSS-wide keywords and never a [`PropertyValue::CustomRef`]. Construct one with
[`ComputedValue::from_concrete`] (which enforces the invariant by returning
[`None`] for a non-concrete input) or with the infallible typed constructors
[`ComputedValue::color`], [`ComputedValue::length`], [`ComputedValue::number`]
and [`ComputedValue::keyword`].

Because the wrapped value may carry `f32` data, this derives [`PartialEq`] but
not [`Eq`]/[`Hash`].
