Checks every collection construct in a `.okui` document against the §2.11
collection rule (spec §2.11, §4.9).

The rule itself is [`check_collection`](crate::validation::check_collection) —
the same function the `okui!` macro is checked with. This type only walks the
syntax tree and attaches a byte offset to each verdict, the mirror of what
[`ExpressionValidator`](crate::validation::ExpressionValidator) does for
§2.10.

Both §2.11 constructs, `<For>` and `<VirtualFor>`, are honoured, so the walk
reports nothing today; it is the seam a future collection constraint would
live in. See the [`collection_rule`](crate::validation::collection_rule)
module docs for how `<VirtualFor>` is realized as a viewport-bounded window
rather than the silent `<For>` it used to compile to.
