A name that matches no element is not an error and not a panic — it stays
pending, is retried after every commit, and is *visible*: `bound_node` answers
`None`, which is how an application finds a misspelled `test-id` instead of
wondering why a button does nothing.
