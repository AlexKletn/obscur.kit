Parses the body of a `{…}` placeholder; `self.at` is just past the `{`.

`in_plural_arm` says whether this placeholder sits inside a `plural` arm.
It is carried in rather than recomputed because it must **survive** a
nested `select`: in `{n, plural, other{{g, select, …{# штук}…}}}` the `#`
is lexically inside the select's arm but semantically still the plural's
number, and dropping the flag at the select would parse it as a literal
`#`.
