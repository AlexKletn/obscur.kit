The outcome of rendering a
[`MountedComponent`](crate::component::MountedComponent) with its error
boundary applied (spec §1.7).

A component's render either produces a view or fails, and a failure is never
just an absence: the boundary has already decided what happens to the
subtree, and the caller is handed both that decision and the structured
failure to report.
