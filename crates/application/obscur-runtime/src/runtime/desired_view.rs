//! The tree a frame reconciles, resolved lazily: [`DesiredView`].

use obscur_view::view::View;

/// The desired tree a frame reconciles, resolved lazily so a reactive frame can
/// build it **after** phase 3 (§3.4 "dispatch input/events") — an input handler
/// that commits state must have run before the view is rebuilt from that state,
/// or its mutation lands a frame late.
pub(crate) enum DesiredView<'a> {
    /// A tree the caller already has in hand (`render_now`, the `&View` frame
    /// loop). Materialization is a no-op borrow.
    Ready(&'a View),
    /// A tree built on demand from current state, called once, at reconcile time
    /// (after input dispatch). The closure was moved out of the runtime by the
    /// reactive caller, so invoking it does not alias `&mut self`.
    Build(&'a mut dyn FnMut() -> View),
}
