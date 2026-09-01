The boundary claims: what an application must be able to do with the facade
as its only dependency.

Compiling this file at all is most of the assertion — see `src/counter.rs`.

Not all of it, though: `xtask consumer` runs `cargo test` on this fixture, not
`cargo build`, so an assertion on a computed value is as load-bearing here as a
resolution error is. The custom-property tests at the end of the file rely on
that — a `var()` that computes to `unset` compiles perfectly well, and only a
measurement catches it.
