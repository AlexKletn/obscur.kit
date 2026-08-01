Checks every authored `<slot …/>` in a `.okui` document against the §2.12
outlet rule.

The rule itself is [`check_outlet`](crate::validation::check_outlet) — the same
function the `okui!` macro is checked with. This type only walks the syntax
tree and attaches a byte offset to each refusal, the mirror of what
[`CollectionValidator`](crate::validation::CollectionValidator) does for §2.11.

Every diagnostic is anchored at the `<slot>` element itself rather than at the
offending attribute. That is deliberate and it is the honest anchor: an outlet
is a two-token construct, and every rule here is about *the outlet* — that it
has no name, that its name is not decidable, that it carries something it
cannot carry. There is no case where the author needs to be pointed inside it.
