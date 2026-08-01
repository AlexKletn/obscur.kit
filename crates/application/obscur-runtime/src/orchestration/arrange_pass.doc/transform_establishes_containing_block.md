Whether `node`'s computed `transform` establishes a containing block for its
absolute- and fixed-positioned descendants (css-transforms-1 §3: "any value
other than `none` for the transform property also causes the element to
establish a containing block for all descendants … used to layout for all of
its absolute-position descendants, fixed-position descendants").

The used padding box is then the positioning reference for those descendants —
including `position: fixed` ones, which would otherwise resolve against the
viewport. An authored transform qualifies even when it resolves to the identity
matrix (e.g. `translateX(0px)`); only the `none` keyword does not.
