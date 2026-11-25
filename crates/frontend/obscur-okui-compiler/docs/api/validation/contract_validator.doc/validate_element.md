Validates `element` against `contract`, returning one diagnostic per
violation (spec §2.13).

The checks are:

- **required props** — every name in
  [`required_props`](ComponentContract::required_props) must appear as an
  attribute on `element`; a missing one yields
  `"component '<c>' requires prop '<p>'"`;
- **unknown attributes** — when the prop surface is closed (the contract
  declares any required or allowed prop), every non-event attribute must
  be a required/allowed prop or a reserved structural attribute
  (`key`/`class`/`id`/`requires`); an unrecognised one yields
  `"unknown attribute '<a>' on component '<c>'"`;
- **unknown events** — when the event surface is closed
  ([`allowed_events`](ComponentContract::allowed_events) non-empty),
  every `on:<event>` binding's event must be listed; an unrecognised one
  yields `"unknown event '<e>' on component '<c>'"`;
- **allowed children** — under [`ChildPolicy::Only`] every child
  *element*'s tag must be listed; a disallowed tag yields
  `"'<tag>' is not an allowed child of '<c>'"`. Under
  [`ChildPolicy::None`] no child is admitted at all, element or text, and
  each one yields `"component '<c>' takes no children: …"`;
- **slots** (§2.12) — a child's `slot="…"` must name a slot the contract
  declares and satisfy that slot's own child rule, and every declared
  slot's cardinality must hold.

Attribute-, event-, child- and slot-assignment diagnostics are anchored at
the offending attribute's or child's byte offset; the required-prop and
cardinality diagnostics are anchored at the element's own.

**String-based method dispatch** (§2.3, §2.13) *is* modelled, at the one
place the `.okui` AST spells it: an `on:<event>` whose value is a string
literal — `on:pressed="handle_submit"` — is a named-method reference, which
§2.3 ("Events are typed. String-based method dispatch is not used.")
forbids. It is refused whether or not the event is declared, because the
*form* is the fault: a handler is bound with `{…}`, never named by string.
See [`check_string_dispatch`](Self::check_string_dispatch).
