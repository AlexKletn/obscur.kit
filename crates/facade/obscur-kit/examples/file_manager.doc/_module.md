A minimal **file manager**, authored in `okui!` markup over real data — the
demo that exercises the declarative entry point end to end.

What it proves, concretely:

* [`App`] boots a window in a handful of lines (see `main`).
* `okui!` markup binds **real Rust values**: `{row.name}` renders the actual
  file name, `class={row.class}` drives the selection highlight, and
  `<For each={rows}>` emits one node per directory entry — a real Rust loop,
  not a template materialized once. (A standalone `.okui` *file* cannot do
  this today: `App::screen` resolves a binding against a `ScreenData` table
  rather than against Rust scope, and its `<For>` is materialized once, so the
  file route reaches *values* and not shape. The inline macro resolves bindings
  in *this* function's scope, which is why the markup lives here.)
* The kit performs no I/O of its own (§5.15): `std::fs` reads the directory,
  the kit only renders. No network, no new dependency.

Rows are **clickable**: a click selects an entry and a double click opens it (a
directory is entered, a file reported in the status line), with the toolbar as a
keyboard-free alternative. A per-row `on:click` cannot carry this — every row the
builder emits lowers to the *same* command symbol, so the symbol alone can never
say which row was hit. The runtime resolves the click to the entry's model index
from its own geometry (the container box, its scroll offset, the row extent) and
calls [`App::on_row_click`]; that is the same "only the runtime holds both halves"
argument as the virtual list itself.

Run it (a display and a GPU adapter are required):
`cargo run -p obscur-kit --example file_manager --features full`
