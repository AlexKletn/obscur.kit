The block-flow orientation both box axes rotate around: [`WritingMode`].

It was a second public entity exported out of `property_id.rs`, which §3.16
forbids, and it is one in its own right:
[`PropertyId`](crate::property::PropertyId) is a *catalogue of
longhands*, while this is the computed **value** of one of them — the one
every logical→physical mapping and every normal-flow stacking decision takes
as an input.
