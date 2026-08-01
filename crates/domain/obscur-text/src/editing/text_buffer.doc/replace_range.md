Splices `new_text` into `range`, records the invertible transaction (with
coalescing per `kind`), and moves the caret to the end of the inserted
text. A no-op when the replacement leaves the bytes unchanged.
