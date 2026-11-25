Runs the WASM lifecycle locally with bounded resources. Host imports are disabled by default.

With `--mock-response`, the CLI enables the version-1 host-call import and
issues a single handle for version 1.0.0 of `cli.mock`, operation `get`.
Its eight-byte big-endian handle is prepended to the initial request. Calls
are dispatched through the normal service codec and session. The fixture
supplies the JSON result, with at most 16 calls across initialization and
dispatch. The final deactivation hook is not a host-call exchange.
