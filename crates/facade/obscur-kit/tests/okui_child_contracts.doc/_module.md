End-to-end: a component contract can say **"no children"**, and both front
ends enforce it (spec §2.13 "allowed children", §2.12 "allowed child
contracts").

# The state that did not exist

The compiler's string contract held children in a `Vec<Box<str>>` whose *empty*
state meant "any child". So a component with nothing to say about children and a
component that takes none were the same value, and the second one is what almost
every shipped component is: `Button`, `Input`, `Checkbox`, `Radio`, `Select` and
the five collection widgets build their children inside their own `view()` from
their own model, and `Image`/`Canvas`/`ForeignSurface` are leaves.
`<Button>OK</Button>` — the props spelling of a control, carrying its content a
second time in a position that spelling does not read — therefore compiled
clean, with no diagnostic. `ChildPolicy` names the three states, and
`no_children()` is the one that was missing.

# Which spelling the rule applies to

`<Button …/>` and `<button>…</button>` share one `ComponentId` —
`name_binding::builtin_component_id` maps `"Button" | "button"` to the widget —
but they are not the same authoring construct, and only the first is held to the
leaf rule:

- a **component instance** `<Button label="OK"/>` is the *props* spelling: it
  states its content as `label`, which the registry requires of it, so a child
  there says the same thing twice and the second spelling is dropped;
- a **base element** `<button class="btn">…</button>` is the *content* spelling:
  it lowers to a node carrying the widget's id, the `button` local name a §4.13
  type selector matches — and **its authored children as its content**, which
  are the only thing that renders.

Since `BuiltinComponents` mounts a `ButtonFactory`, both spellings reach one
mounted component — and it renders the §2.12 default-slot outlet, so the
base element's children are projected back and the component instance's `label`
is the outlet's fallback. The compiler rule is therefore about which spelling
an author is writing, not about what the runtime could do with the other.

Contracts are keyed by the authored tag, which is what keeps the two apart. The
`checks/facade-consumer` fixture is exactly
`<button class="btn" on:click={increment}><span>"press"</span></button>`, and it
must keep compiling.
