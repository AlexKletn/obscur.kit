The set of computed properties whose value differs between two
[`ComputedStyle`]s, plus whether any custom property changed.

A property counts as changed when it was added, removed, or has a differing
computed value. The list returned by [`StyleDiff::changed_properties`] is
sorted (by canonical property name) so equal diffs compare equal and output is
reproducible.

Because the comparison is over `f32`-bearing values, this derives
[`PartialEq`] but not [`Eq`]/[`Hash`].
