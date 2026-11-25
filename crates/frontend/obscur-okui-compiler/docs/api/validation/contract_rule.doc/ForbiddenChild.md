A child element written inside a component that admits **no** children
([`ChildPolicy::None`](crate::validation::ChildPolicy::None)).

Distinct from [`DisallowedChild`](Self::DisallowedChild) because the fault
is different and so is the fix: there is no allowed tag to reach for, and
the diagnostic must say so instead of implying a list the author could
consult.
