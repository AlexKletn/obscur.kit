A component's owned, mutable local view state.

Per spec §2.1/§10 a component *may* own local view state; this is that
primitive. Because the component owns the value outright, mutation is
expressed through `&mut self` — plain Rust ownership — rather than interior
mutability (`RefCell`/`Rc`) or any global store. Every mutation advances a
[`StateGeneration`] so downstream consumers can cheaply detect the change.

State only ever flows *out* as a read-only [`ReadSignal`] via
[`LocalState::read`]; there is no shared writable handle, keeping the
"values flow down, actions flow up" discipline intact.
