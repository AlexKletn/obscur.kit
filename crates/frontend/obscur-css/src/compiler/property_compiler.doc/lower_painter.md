Lowers `-obscur-painter`: an identifier or a quoted string naming a registered
painter, or the keyword `none`.

The name is kept as TEXT rather than interned. A `Keyword`'s symbol is minted
by the interner of whichever stylesheet happened to be compiling, so the same
word in two sheets is two different symbols and neither matches a registry key.
