The [`ValueParser`] and its parsed component-value AST.

A declaration value is modelled as the CSS *component value* list (spec
§4.13): a flat [`Vec<ParsedValue>`] of space-separated components in which
commas and operators survive as their own [`ParsedValue::Comma`] /
[`ParsedValue::Delimiter`] components. Functions nest their arguments as a
child component list. Nothing is evaluated here — `calc()` and friends keep
their raw operands so the compiler can lower or fold them later.
