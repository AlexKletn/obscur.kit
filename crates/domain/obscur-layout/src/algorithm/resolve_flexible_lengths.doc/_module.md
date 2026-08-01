*Resolve Flexible Lengths* — css-flexbox-1 §9.7.

The freeze loop, and nothing else. It is one entity by the plainest test
there is: one function, eight numeric inputs, one vector out, and internals —
frozen sets, signed violations, the scaled shrink factor, the `Σfactors < 1`
cap — that no caller ever names. [`super::FlexLayout`] asks for the line's
final main sizes and learns none of it.

Two of the loop's guards are there because a layout that does not terminate
does not merely draw wrongly; both are documented at the point they act.
