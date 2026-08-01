Parses and compiles `source` with no state declared — for a screen whose only
bindings are in element bodies.

An **attribute** binding (`class={theme}`) references a name, and §2.13 makes
an unknown name a compile error, so a screen that binds an attribute must
declare it: use [`compile_with`](Self::compile_with) and bind the name in the
table first.

# Errors

See [`compile_with`](Self::compile_with).
