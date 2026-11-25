Returns the byte index of the extension's dot in `path`'s final segment, or
`None` when that segment has no extension.

A leading dot in the segment is a dotfile, not an extension (`.gitignore`),
and a dot in a parent directory is not one either — hence the search is
confined to the final segment.
