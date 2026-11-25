The type-checking helper for props and simple expressions (spec §2.9,
§2.10).

Because the syntax layer captures each `.okui` expression as raw text (there
is no expression AST yet), this checker works on the pieces the resolution
and lowering steps can already produce: it classifies a *literal token's*
text, types a *reference* to a resolved [`Symbol`], and applies the
[`ValueType`] assignability algebra to check props and conditions. Full
expression-tree inference awaits an expression parser and is out of scope
here.
