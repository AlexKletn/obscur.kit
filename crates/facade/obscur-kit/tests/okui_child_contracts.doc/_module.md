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
`<Button>OK</Button>` therefore compiled into a text run nothing mounts —
meaningless markup with no diagnostic. `ChildPolicy` names the three states, and
`no_children()` is the one that was missing.

# Which spelling the rule applies to

`<Button …/>` and `<button>…</button>` share one `ComponentId` —
`name_binding::builtin_component_id` maps `"Button" | "button"` to the widget —
but they are not the same authoring construct, and only the first is held to the
leaf rule:

- a **component instance** `<Button label="OK"/>` renders the widget's own
  `view()`, which never looks at authored children, so a child there is dead
  markup;
- a **base element** `<button class="btn">…</button>` mounts no widget view at
  all: it lowers to a node carrying the widget's id, the `button` local name a
  §4.13 type selector matches — and **its authored children as its content**,
  which are the only thing that renders.

Contracts are keyed by the authored tag, which is what keeps the two apart. The
`checks/facade-consumer` fixture is exactly
`<button class="btn" on:click={increment}><span>"press"</span></button>`, and it
must keep compiling.
