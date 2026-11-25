Returns the contracts for every shipped widget (spec §2.6, §2.13), keyed
later by tag name exactly like [`builtin_contracts`].

All twelve widget tags already bind to a real `ComponentId` through
`obscur_ui_ir::name_binding`, but none carried a contract — so
`<Button laebl="OK"/>` compiled clean. These close that hole for the widgets
that have an authored surface.

# Where the surface comes from

Same rule, and same reason, as the primitives above: `obscur-okui-compiler`
is a *frontend* crate and cannot depend on the components tier, so the
surfaces are mirrored rather than imported. The widgets' own prop symbols
could not drive this even if they were reachable — they are private, and they
name the state a widget *writes out* (`role`, `pressed`, `selected`), not the
attributes an author *writes in*. The authoritative string surface is
therefore each widget's §2.6 builder API: a positional `new()` argument with
no default is a **required** prop, and each fluent setter is an **allowed**
one.

# Deliberate omissions

- **`command` is not a prop.** Every interactive widget takes
  `command: InternedSymbol` in its constructor, but the view binds it as the
  event *handler*, so it is authored as `on:click=submit`, never as an
  attribute.
- **Collection widgets declare no props.** `Menu`, `Tabs`, `Tree`, `Table`
  and `Docking` take their content as typed Rust values (`MenuItem`,
  `TreeNode`, `Column`, `DockPanel`, labels) with no string spelling at all.
  Inventing an `<Option>` / `<Tab>` / `<Column>` child vocabulary would be new
  design, not mirroring, so their *prop* surface stays **open** (as
  `Container`/`Text` do) and stays unchecked until that vocabulary is actually
  specified. Their *child* surface is a different question, answered below.

# "No children", and which spelling it applies to

Only `Dialog` accepts authored children; every other widget builds its
children inside its own `view()` from its own model, so the children an author
writes reach no tree at all. That used to be inexpressible — an empty
allowed-children list meant *any* child — so `<Button>OK</Button>` compiled
clean, saying the button's content twice in a position this spelling does not
read (`<Button/>` states it as the `label` these contracts require; the base
element `<button>…</button>` is the spelling whose content is its children). Each of those eleven widgets now declares
[`no_children`](ComponentContract::no_children)
([`ChildPolicy::None`](crate::validation::ChildPolicy::None)), including the
five collection widgets: `<Tabs><Tab/></Tabs>` mounted an empty widget in
silence, and a compile error naming the tag is what a reader expects there.
The day a `<Tab>`/`<Column>` vocabulary *is* specified, the policy for that
widget becomes `Only([…])` and this paragraph is what to revisit.

**The claim is about the component tag `<Button …/>`, not about the base
element `<button>`.** The two spellings share one
[`ComponentId`](obscur_identity::ComponentId) —
`name_binding::builtin_component_id` maps `"Button" | "button"` to the widget —
but they do *not* share a contract, because this registry is keyed by the tag
the author wrote. That is exactly right, and not an accident of the keying:

- `<Button label="OK"/>` is a **component instance**. Its label, its pressed
  state and its focus ring come from the widget's own `view()`, which ignores
  whatever children the markup carried. A child there is dead markup.
- `<button class="btn">…</button>` is a **base element**, as in HTML. Nothing
  mounts the widget's view for it: it lowers to a node with the widget's id, its
  `button` local name (so `button { … }` styles it, spec §4.13) and **its
  authored children as its content**. Those children are the only thing that
  renders, so forbidding them would break the base element outright — the
  `checks/facade-consumer` fixture is exactly
  `<button class="btn" on:click={increment}><span>"press"</span></button>`.

So: a lowercase element spelling has no contract registered here and is left
alone, and only the PascalCase component position is held to the leaf rule.
