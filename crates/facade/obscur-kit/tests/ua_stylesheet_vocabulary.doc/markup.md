Compiles a `.okui` source string to a `View` through the interpreted front end.

A string rather than the `okui!` macro, because the markup-only vocabulary is a
list this file loops over — a macro would need one call site per tag.
