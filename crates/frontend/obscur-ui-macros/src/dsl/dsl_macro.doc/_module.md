`DslMacro`: expansion logic for the TSX-like Rust DSL (spec §2.8).

This module parses a JSX/TSX-like element tree into a
[`UiIrDocument`] **while the macro runs**. The document is not emitted or
interpreted: `okui!` hands it straight to `ViewCodegen`, which lowers it to
ordinary `View`-building Rust, so no IR survives into the built crate.

# Grammar

- Element: `<Tag ...attrs> ...children </Tag>` or self-closing
  `<Tag ...attrs />`.
- Tag naming (TSX convention): an UPPERCASE-initial identifier (`Button`) is
  a component, a lowercase-initial identifier (`div`) is an element.
- Attributes:
  - An attribute (or event) **name** is `ident ('-' ident)*`, joined into one
    hyphenated string — see [`parse_dashed_name`] for why the join is a parse
    step and not a lexical one.
  - `name="literal"` → [`UiIrProperty::text`](::obscur_ui_ir::document::UiIrProperty).
  - `name={expr}` → a literal when `expr` is a literal, a
    [`binding`](::obscur_ui_ir::document::UiIrProperty::binding) when it is a
    single ident or dotted field-access path, else a `binding` whose path is
    the stringified expression.
  - `name` (bare) → boolean-true literal property.
  - `on:click={handler}` → an [`UiIrEvent`](::obscur_ui_ir::document::UiIrEvent).
- Children (in order): nested elements, bare string literals (text nodes),
  and `{expr}` interpolations.

# Representation choices

- Binding/handler paths are normalised to a dotted, space-free string:
  `{user.name}` becomes the path `"user.name"` (not `"user . name"`).
- Interpolated `{expr}` children lower to an
  [`interpolation`](::obscur_ui_ir::document::UiIrNode::interpolation) node
  carrying the expression — the *same* node the `.okui` file lowering builds,
  so `ViewCodegen` emits the same text node from either front end. They used
  to disagree: this one built a literal text node holding the path's spelling,
  so `<div>{user.name}</div>` rendered the characters `user.name`.
- A lone top-level element is emitted as the document root directly; any
  other top level (multiple siblings, or a single text/interpolation node)
  is wrapped in a [`fragment`](::obscur_ui_ir::document::UiIrNode::fragment)
  root.

# Id scheme

Every node is assigned a stable sequential `u64` id during a single
pre-order walk (root = `0`, incrementing).
