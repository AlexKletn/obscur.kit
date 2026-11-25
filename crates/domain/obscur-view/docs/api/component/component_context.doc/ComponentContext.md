The context threaded through [`render`](crate::component::Component::render)
and the lifecycle hooks (spec §2.1).

It is deliberately minimal and *honest*: there is no reactive runtime yet,
so the context does not dispatch actions, resolve capabilities, or reach any
global store (there is none — local view state lives in the component's own
[`LocalState`](crate::state::LocalState) fields). It only exposes what a
component legitimately needs during a single render/lifecycle call:

* the component's [`SubscriptionScope`], so it can open scoped subscriptions
  that are automatically released when the scope is dropped;
* the current [`LifecycleState`], so a hook can branch on the phase;
* a way to *request* a re-render ([`request_update`](Self::request_update)),
  which merely sets a flag the runtime observes later via
  [`needs_update`](Self::needs_update) — the context never renders anything
  itself.

# Borrows

`ComponentContext<'a>` borrows the instance's subscription scope and its
"needs update" flag mutably for `'a`; it owns no state of its own. Because
it holds `&mut` borrows it is invariant in `'a` and therefore not `Clone`.
