End-to-end: `.okui` **conditions decide** and **bindings carry values**
(spec §2.10, §2.11).

Both were inert, and inert in the worst way — silently wrong rather than
unimplemented. `<If cond={open}>` rendered its body whatever `open` was,
because the conditional was a transparent passthrough that spliced its
children. And a binding emitted its own *spelling*: `value={label}` produced
the literal text `"label"`, so a dynamic document rendered its own source code.

The fix needs no evaluator. A binding is a **Rust path**, emitted verbatim into
the scope the macro expands in, and a condition is a real `if`. Everything
below is checked against a `View` built by the actual `okui!` macro, with real
locals in scope — the same thing an application writes.
