The parameters half of §7.2's *key + parameters* abstraction: values bound to
names.

Arguments are **named**, not positional, because word order is exactly what
translation changes. A positional `{0} {1}` forces every language to keep
English's order or silently swap meanings; `{name} {count}` lets a translation
put them wherever its grammar wants.

Backed by a flat vector: a message has a handful of arguments, and a linear
scan over three of them beats hashing every one of them.
