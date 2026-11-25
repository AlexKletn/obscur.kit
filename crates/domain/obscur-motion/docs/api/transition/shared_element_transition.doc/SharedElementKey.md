A stable tag identifying the *same* element across two views (spec §4.17
"stable identity").

When a shared-element transition runs, the runtime pairs elements by matching
keys: an element tagged `SharedElementKey("avatar")` in the old view morphs
into the element with the same key in the new view. This is a *local* newtype
— the motion crate owns and compares these tags itself and does not depend on
any shared identity crate.
