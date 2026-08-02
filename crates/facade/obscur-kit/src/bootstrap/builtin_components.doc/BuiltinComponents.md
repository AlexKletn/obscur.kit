The registrations that turn the shipped widgets from *builders* into
*mountable components* — the answer to "who registers a factory for the
built-in widget ids".

This lives in the facade, and nowhere else could: `register_component` is a
runtime call and the factories are components-tier types, so the facade is
the only crate that may name both. The list is deliberately a value a host
can read (see [`registrations`](Self::registrations)) rather than a private
loop inside a bootstrap method: a host that wants a different `<checkbox>`
needs to know which id is already taken.

**Registering a built-in changes what existing markup renders.** Before,
`<checkbox label="Agree" checked/>` was an element with the right id, the
right local name and no contents. After, it is the control: the
`.checkbox-box` indicator sub-part the UA sheet fills, and the label. The
same for `<radio label="One" checked/>` and its `.radio-dot`, and for
`<select options="Alpha, Beta" open/>` and its `.select-header` /
`.select-list`. Nothing about the host node changes — the props, the
reserved `:checked` / `:disabled` attributes and the event bindings are the
*authored* node's and stay exactly where the cascade and hit-testing
already read them.

The list is short on purpose. A widget appears here only once it has a
props struct, a `Component` and a factory; `Menu`, `Dialog`, `Tabs`,
`List`, `Tree` and `Docking` are absent for a different reason — they have
no lowercase tag and no UA rule, so a component for them would answer no
question anything is asking.

Four of the five **own their content**: a checkbox's indicator, a radio's
dot, a select's header, arrow, list and rows, and a field's editable run are
the control's, not the author's, so each mounts without any §2.12 slot
machinery. `Button` is the one that does not — `<button>Save</button>` puts
the label in the node's *children* — and it is here because the slot that
answers it now exists: `ButtonComponent` renders the default-slot outlet, so
the author's content is projected back into the position the control names
for it, and `label="…"` is the fallback for markup that supplies none.

`Table` is deliberately still absent, and not for want of a mechanism. Its
rows and cells are `tr`/`td`/`th` elements the author writes or the Rust
builder generates from data, and a component mounted on it could only render
the outlet that hands them straight back — a mount that changes nothing,
which is surface with nothing asking for it. `<table>` stays an author-owned
base element.
