The winning **cascaded value** for each property that participated in the
cascade for a node (§4.14).

This is the output of [`crate::cascade::Cascade::resolve`]: for every
[`PropertyId`] that had at least one matching [`crate::cascade::Declaration`],
it holds the value of the declaration with the greatest
[`crate::cascade::CascadePriority`].

These are *cascaded* values, **not** computed values. Resolving CSS-wide
keywords (`inherit` / `initial` / `unset`), applying inheritance from the
parent, and substituting `var()` references all happen in a later sub-step
(computed style); none of that is applied here.
