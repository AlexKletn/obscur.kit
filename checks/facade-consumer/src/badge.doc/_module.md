A component an application *authors*: the declaration, the render, the
registration, and the tag — with the facade as the only dependency.

# Why this file exists

`src/counter.rs` guards what `okui!` emits. Nothing guarded what
`#[derive(Contract)]` emits, and it emitted `::obscur_view::…` and
`::obscur_identity::…` long after the macro stopped: audit finding 2-2, the
residue of 1-A. The two `publish = false` crates those paths name are not in
this crate's extern prelude and never can be, so an application that declared
a component got

```text
error[E0433]: cannot find `obscur_view` in the crate root
```

from the derive alone — the mechanism §2.13 makes a component *out of*. The
defect survived a whole fix campaign because the fixture written to catch that
class of defect did not author a component; the hole in the guard was the same
shape as the defect. So: this crate now authors one, and compiling it is most
of the assertion.

# Why the declaration is as broad as it is

Each line of `BadgeProps` is here because it makes the derive emit a *path*
the others do not: a required prop and an `Option` prop reach different
decoders, `#[contract(default)]` a third, `#[contract(event)]` the event
contract, and a `slot(...)` the slot contract and its `ChildContract`. A
resolution failure in any one of them is a compile error in this crate, which
is the only place it is visible at all.

The component is nonetheless a real one, not a syntax sampler: it renders its
decoded props and gives its authored children a place to land.
