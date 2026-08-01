Read-only snapshot of a value together with the [`StateGeneration`] at which
it was produced.

`ReadSignal<T>` is the "values flow down" half of the data-flow model: it
hands consumers the current value and a change token, but exposes **no**
mutation API, so there is no magic two-way binding. A component obtains one
from its own [`LocalState`](crate::state::LocalState) (or, later, from an
upstream source) and passes it down as a prop.

Because there is no reactive runtime yet, a `ReadSignal` is a lightweight,
self-contained snapshot rather than a live handle: it carries the value by
ownership and its generation by value. Re-reading an updated source means
producing a fresh snapshot with a newer generation; a consumer detects the
change with [`ReadSignal::changed_since`].
