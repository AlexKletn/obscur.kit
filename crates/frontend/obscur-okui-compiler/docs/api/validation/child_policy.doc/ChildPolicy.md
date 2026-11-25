Which children a [`ComponentContract`](super::ComponentContract) admits,
named by tag.

# Why this is an enum and not a list

It used to be one field — `allowed_children: Vec<Box<str>>` — whose
*emptiness* meant "any child". That gave the contract model two states where
§2.13 needs three, and the missing one was the state almost every shipped
component wanted: `Button`, `Input`, `Checkbox`, `Radio`, `Select` and the
collection widgets synthesise their content inside their own `view()` from
their own model, and `Image`/`Canvas`/`ForeignSurface` are leaves. So
`<Button>OK</Button>` — the *props* spelling of a control, carrying its content
a second time in a position that spelling does not read — compiled clean,
because the only way to say "no children" was the very same empty list that
already meant "any".

Overloading emptiness a second time (an empty list meaning *none* in some
contexts) would have made the field's meaning depend on who was reading it.
Naming the three states removes the question: **emptiness never means
"unrestricted" anywhere in this type**, and [`Only`](Self::Only) with an empty
list therefore admits nothing, exactly as [`None`](Self::None) does.

# The mirror

The three cases are `obscur_view::contract::ChildrenSurface`'s and
`obscur_view::contract::ChildContract`'s, deliberately spelled the same:
`ChildContract` decides membership by
[`ComponentId`](obscur_identity::ComponentId), `ChildrenSurface` by tag name
for tooling, and this by tag name for the `.okui` front ends. The bridge in
[`surface_bridge`](super) crosses `ChildrenSurface` into this, and it used to
have to flatten `None` into `Any` because this type could not hold it — the
loss the enum removes.

# Content is not a tag

A text child or an `{expr}` interpolation has no tag, so a *list* of allowed
tags cannot speak about it: [`Only`](Self::Only) restricts which element tags
may appear and leaves content alone, which is what "allowed children" means as
a list of names. [`None`](Self::None) is a claim of a different kind — that the
position is a leaf — and content violates it exactly as an element does. See
[`admits_content`](Self::admits_content).
