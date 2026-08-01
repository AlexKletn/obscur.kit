Typed AST wrapper over a reusable template body (spec §2.12).

[`cast`](Self::cast) accepts a dedicated [`Template`](SyntaxKind::Template)
node as well as a whole [`Document`](SyntaxKind::Document): because the parser
emits `Template` nodes only rarely, an entire document can be treated as a
template body. Its top-level [`Element`]s are its [`roots`](Self::roots).

# This type is dead, and its slot model was the wrong way round

The parser never emits a [`Template`](SyntaxKind::Template) node, and nothing
casts one; the `.okui` front end has no component-authoring surface for a
template to be the body *of*. That is why §2.12's slots are a *consumer*-side
construct here — see `obscur_okui_compiler::validation::contract_rule`.

[`slot_names`](Self::slot_names) collects the `slot="name"` attributes on the
subtree's descendants. Those are slot **assignments** — each says which of its
*parent component's* declared slots that child goes in — not slots the subtree
declares, which is what this type's name suggests and what it does not do. A
declaration lives on a component's contract
(`obscur_view::contract::SlotSurface`), which is string data, not syntax.
