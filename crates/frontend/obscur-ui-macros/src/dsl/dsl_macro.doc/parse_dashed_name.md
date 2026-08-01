Parses an attribute (or event) name: `ident ('-' ident)*`, joined into the one
hyphenated string both front ends spell a reserved §7.5 identifier with.

# Why the join has to happen at all

`okui!` is a proc-macro over **Rust** tokens, where `-` is punctuation and can
never be part of an identifier, so `test-id="a"` arrived here as three tokens
and died on `Ident::parse_any` with `expected ident`. The `.okui` file front
end has its own lexer, whose identifier rule admits `-`
(`obscur_okui_syntax`'s `is_ident_continue`), so it accepted the very same
markup. The two front ends therefore disagreed about what an attribute may be
*called* — and §7.5's `test-id` / `automation-id`, which
`App::on_command_at` and `App::add_event_listener` name their target with,
were authorable from the fallback front end only. `data-*` and every future
hyphenated reserved name were unreachable the same way.

# Why joining here is safe and joining anywhere else would not be

The join is confined to **name position**: at the head of an attribute inside
an opening tag, and immediately after `on:`. No expression can appear in
either place, so a `-` there has no other meaning to steal. Subtraction lives
inside `{ … }`, which arrives as a single brace group that `syn::Expr` parses
and this function never looks into, so `{a - b}` means exactly what it meant
before (pinned by `subtraction_is_still_subtraction_everywhere`).

A **tag** is deliberately not joined. The file lexer would admit `<my-tag>`
the same way it admits `my-attr`, but a hyphenated tag names no primitive, no
widget and no component — it would hash to a component id nothing registers —
so widening this front end to accept one would buy a second spelling for
"unmounted box". It is recorded rather than closed; see
`the_names_only_the_file_front_end_admits_are_recorded_not_imagined` in
`obscur-kit`'s `okui_macro.rs` for the rest of that ledger.

# Why adjacency is not checked

It cannot be, from here. `Punct::spacing()` reports `Joint` only when the
*next character is itself punctuation* (`proc_macro2`'s `parse::punct`), so
the `-` in `test-id` and the one in `test - id` are both `Alone`: spacing
distinguishes `-=` from `-`, not `a-b` from `a - b`. `Span::source_text()`
could see the difference in a real expansion but answers `None` under
`quote!`, i.e. in every unit test below. Nothing is lost by not checking:
the grammar position is unambiguous, and a name a stray space produced is
still a name that has to match a reserved spelling or a contract to mean
anything.
