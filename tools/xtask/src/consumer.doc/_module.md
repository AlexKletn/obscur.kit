The consumer guard: does a crate on the far side of the facade build?

Two profiles, because one was not enough. `checks/facade-consumer` stands
outside the facade and opens a **window**; `checks/layer-consumer` stands
outside it and opens a **desktop surface**, in a feature set with no windowing
backend at all. The two runners diverge after `AppParts::boot`, and each profile
covers a half the other never reaches — `docs/layer-consumer-audit.md` is what
turned up the first time anything covered the second one.
