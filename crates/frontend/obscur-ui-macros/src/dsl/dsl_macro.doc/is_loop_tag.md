Whether `tag` names a collection construct rather than an element
(spec §2.11).

`VirtualFor` is classified with `For` — §2.11 says it is a collection, and
misclassifying it as an element would render a node called `VirtualFor`. The
two are *not* emitted alike, though: a `<For>` lowers to a loop node the
codegen emits as a real per-item `for`, while a `<VirtualFor>` keeps its tag
so the codegen/bridge lower it to a virtual-collection marker the runtime
realizes into a viewport-bounded window (spec §4.9). This predicate keeps
both names so each reads as "a loop", the tag downstream then telling them
apart.
