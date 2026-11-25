The §2.13/§2.12 contract rule itself, stated over plain data so **both**
front ends decide a document's legality the same way.

# Why the rule is not stated over the AST

There are two authoring surfaces and only one of them has a `.okui` syntax
tree. The file path parses to [`Element`](obscur_okui_syntax::ast::Element)
nodes; the `okui!` macro parses `TokenStream`s with `syn` and never builds
one. A rule written against either input can only ever check that input —
which is exactly how the macro came to run **no** contract validation at all
while the file path ran this one over every element it parsed.

So the rule takes an [`ElementUse`]: the tag, the attribute names, and the
members, with no syntax in it. Each front end describes its own instance and
attaches its own location to the resulting [`ContractViolation`]s — a byte
offset for the file path, a `Span` for the macro. One rule, two adapters:
the same shape `check_expression` already gave §2.10.

# Members, not children

A component's *members* are the nodes that actually land in its child list,
which is not the same as its direct children: `<If>` and `<For>` splice their
bodies into the enclosing sink (see `ViewCodegen::emit_into`), so
`<Card><If cond={x}><h1 slot="header"/></If></Card>` puts the `h1` in Card's
children, not the `If`'s. The rule therefore looks through control flow — and
records what that costs it: a member behind an `<If>` may not be there, and a
member inside a `<For>` may be there many times. [`MemberArity`] carries that,
and cardinality reports only what is *certainly* wrong.
