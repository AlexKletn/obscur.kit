One mounted component instance: its bookkeeping record plus the erased value
the runtime actually renders (spec §2.1).

The two halves come from the same factory but are deliberately distinct:
[`ComponentInstance`] is plain, inspectable data (identity, lifecycle state,
subscription scope), while the [`ErasedComponent`] owns the author's
[`LocalState`](obscur_view::state::LocalState) fields. Keeping *this* value
alive across frames is precisely what preserves a component's local state;
dropping it releases that state and the instance's scoped subscriptions,
which is the spec's "automatic cleanup".

# `needs_update`

[`ComponentContext::request_update`] writes through a `&mut bool` the caller
owns — this struct is that owner. The flag is the component's way of saying
"render me again"; the runtime reads it after each render and clears it when
it acts, so a request is never serviced twice.

# The error boundary

Every component is an error boundary by default (spec §1.7), so every
mounted instance owns one — built from the author's
[`error_boundary`](obscur_view::component::Component::error_boundary) policy
and [`fallback`](obscur_view::component::Component::fallback) when it mounts.
[`render`](Self::render) goes *through* it: a failed render is contained,
retried or propagated here, never returned raw to the caller.
