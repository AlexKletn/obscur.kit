Whether `name` is an `on:` event the runtime dispatches.

`false` means [`event_symbol`] will hash the name into a symbol that binds a
command nothing ever fires. That is a legitimate thing to build on — a host may
dispatch such a command itself through `UiRuntime::dispatch` — so this answers
the question rather than deciding what to do about it.
