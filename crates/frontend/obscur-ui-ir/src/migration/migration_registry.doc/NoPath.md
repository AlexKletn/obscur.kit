No chain of registered migrations leads from `from` to `to`. Raised when
some intermediate version has no outgoing migration, or when the search
exceeds its iteration bound (a cyclic or non-advancing chain).
