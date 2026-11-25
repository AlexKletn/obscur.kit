The [`SelectorParser`] and its parsed selector AST (spec §4.13).

Supports the toolkit's selector surface: type/universal, class, id,
attribute (`[a]`, `[a=b]`, `[a^=b]`, `[a$=b]`, `[a*=b]`, `[a~=b]`, `[a|=b]`),
pseudo-class (including functional forms whose argument is captured as raw
text), pseudo-element (`::before`/`::after`), and the four combinators. No
specificity is computed here — that is the compiler's responsibility.
