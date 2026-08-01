A view built by the `okui!` macro over a value from the caller's own scope.

This function is the whole of audit finding 1-A: it did not compile, because
the expansion named `::obscur_view` and `::obscur_identity` — crates this one
does not (and must not) depend on.

It carries audit finding 2-3 too. `test-id="count-row"` is three tokens to
Rust (`test`, `-`, `id`), so this markup was `expected ident` and §7.5's
identifiers — the only way an application names a node to
`App::on_command_at` before the first frame — were authorable in a `.okui`
file and nowhere else. The hyphenated names below are the assertion; the test
only has to look them up.
