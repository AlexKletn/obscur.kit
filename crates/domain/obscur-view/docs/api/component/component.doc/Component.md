The author-facing contract every component type implements (spec §2.1).

A component:

* receives typed inputs as [`Props`](Self::Props) — values flow *down*
  (spec §3.1); there is no two-way binding;
* produces a declarative [`View`] from [`render`](Self::render);
* may hold *optional local view state* as
  [`LocalState<T>`](crate::state::LocalState) **fields on the implementor**.
  The toolkit provides no global store; a component owns its own state and
  the runtime never reaches in to mutate it (spec §2.1, §3.1);
* may open *scoped subscriptions* and *scoped async tasks* through the
  [`ComponentContext`], all automatically cleaned up when the instance's
  scope drops;
* has minimal lifecycle hooks ([`on_mount`](Self::on_mount),
  [`on_update`](Self::on_update), [`on_unmount`](Self::on_unmount)) whose
  default bodies do nothing;
* is an error boundary by default, customizable via
  [`error_boundary`](Self::error_boundary).

# Not object-safe by design

`Component` has an associated `Props` type and a generic-free but
`Self`-typed API, so it is intentionally **not** object-safe. Heterogeneous
storage and type-erasure (holding many differently-typed components behind
one handle, dispatching by [`ComponentId`](obscur_identity::ComponentId)) is
the concern of the component registry and runtime in later sub-steps — not of
this authoring trait. Keeping props strongly typed here is the whole point;
erasing them is a separate, deliberate boundary crossing.
