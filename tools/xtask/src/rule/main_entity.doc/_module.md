Rule: one file does not export several main entities without an explicit
exception marker (spec §3.16, last bullet; §3.1 "одна основная сущность —
один файл").

# What counts as a main entity

A **candidate** is a `pub` or `pub(crate)` `struct`, `enum` or `trait`
declared at *file scope* — depth zero, so nothing inside a `mod`, an `fn` or
a `#[cfg(test)] mod tests` block can be one.

A candidate is a **main entity** when it also carries its own behaviour:
[`MIN_BEHAVIOUR_LINES`] or more lines of inherent `impl` (or, for a trait,
of trait body). That qualifier is not a softening of the spec, it is what
makes the check say anything about *this* workspace, and the number that
justifies it was measured rather than guessed:

- **194 of 1,813 files** declare more than one `pub`/`pub(crate)` type at
  file scope;
- **169** of those are re-exported by name from their `mod.rs`, so the bare
  reading of "экспортирует несколько сущностей" catches them too;
- **4** hold more than one type with real behaviour.

The 190 in between are §3.1-legal and always were: the rule admits "одну
основную публичную или доменно значимую сущность, её `impl`, приватные
локальные функции" — and an entity's own *vocabulary* is not a second
entity. `font_face.rs` holds `FontFace` and the `FontMetrics` its methods
return; `table.rs` holds `Table` with the `Column` and `SortDirection` a
table is described in; every `port/*.rs` holds a trait and the `Null…` /
`Mock…` stub that exists only to implement it. A check that reported all 194
would report nothing: it would be turned off within a week, and the four
files that genuinely carry two entities would be invisible inside the noise.

Behaviour is the discriminator because it is what the decomposition campaign
actually found each time it hit this violation. `paragraph_layout.rs` was not
at fault for exporting `LineStrut` beside `ParagraphLayout` because
`LineStrut` was `pub` — it was at fault because `LineStrut` had its own
CSS 2.1 §10.8.1 line-box formula, its own `impl`, and an interface far
narrower than its internals. See `docs/runtime-decomposition.md`.

# The exception marker

§3.16 asks for the check "без явного exception marker", so a file may claim
the exception by carrying [`MARKER`] in a comment:

```text
// xtask:allow(multiple-entities) — the three tightly-coupled pieces of one bridge
```

A claim is only honoured when the file *and* the [`KNOWN`] ledger both carry
it, and the rule fails in every direction: a marker with no ledger entry, a
ledger entry with no marker, an entry whose file has since come down to one
main entity, and an entry naming a file that no longer exists. That is the
same two-way discipline `file_size.rs` uses, and for the same reason — an
exception that can go stale silently stops being an exception and becomes a
hole.
