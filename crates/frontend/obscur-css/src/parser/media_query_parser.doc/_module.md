The [`MediaQueryParser`] and its parsed media-query AST (spec §4.13).

Parses an `@media` prelude: qualifiers (`not`/`only`), media types
(`screen`, `all`, `print`), `and`/`or` connectors, colon feature queries
(`(min-width: 600px)`, `(prefers-color-scheme: dark)`), boolean features
(`(hover)`), and range features (`(width >= 600px)`). Feature values reuse
the crate's [`ParsedValue`] model.
