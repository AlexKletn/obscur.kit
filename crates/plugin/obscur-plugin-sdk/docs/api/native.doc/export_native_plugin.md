Exports the C lifecycle entry point once per plugin library. Both callback
paths must accept an `AbiHandle` and return an `AbiStatus`. The generated
wrappers catch unwinding Rust panics before returning to the host.

Pass `host = binding_function` to also export the host table binding. The

binding receives the instance handle and its version-1 host callback table.
