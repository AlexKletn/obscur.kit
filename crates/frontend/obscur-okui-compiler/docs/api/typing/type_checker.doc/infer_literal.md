Infers the type of a literal token from its raw `text` (spec §2.10).

The classification is:

| `text`                     | inferred type                    |
|----------------------------|----------------------------------|
| `true` / `false`           | constant [`Bool`](ValueType::Bool) |
| `[+-]?[0-9]+`              | constant [`Int`](ValueType::Int) |
| `[+-]?[0-9]+.[0-9]+`      | constant [`Float`](ValueType::Float) |
| `"..."` (double-quoted)    | constant [`Text`](ValueType::Text) |
| `null` / `none`            | constant `Optional<Unknown>`     |
| anything else              | [`unknown`](ExpressionType::unknown) |

A non-literal (for example an identifier or a compound expression) falls
through to [`ExpressionType::unknown`]; its real type comes from the
symbol table via [`TypeChecker::infer_reference`].
