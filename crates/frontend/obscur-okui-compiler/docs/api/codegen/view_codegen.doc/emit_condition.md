Emits an expression in boolean position.

A binding is its Rust path, verbatim — `cond={open}` is `if open`. A literal
`true`/`false` is itself. Anything else (a number, a string, a call) is not a
condition, and emits `false` rather than a type error inside generated code
the author cannot see.
