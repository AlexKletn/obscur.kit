The type vocabulary and type algebra for props and simple expressions
(spec §2.9 typed props/events, §2.10 expressions).

This module supplies the DSL's [`ValueType`] vocabulary and its
assignability algebra, the inferred [`ExpressionType`] of a value, the
[`TypeChecker`] helper that classifies literals and checks props and
conditions, and the [`TypeError`] it reports. Because the syntax layer
captures each expression as raw text, this is the type *vocabulary* and
*algebra* rather than a full expression-tree inference — that awaits an
expression parser.
