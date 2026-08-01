The [`ContainerQueryParser`] and its parsed container-query AST (spec §4.13).

Parses an `@container` prelude: an optional container name followed by a
feature condition list (`(min-width: 400px)`, `(inline-size > 200px)`). The
feature/condition grammar is shared with [`super::media_query_parser`].
