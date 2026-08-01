Builds a `name -> 1-based line number` map from a container's
`grid-template-columns` / `grid-template-rows` line names.

The track list's `line_names[i]` bucket holds the names on grid line `i + 1`
(line `0` is the line before the first track). A name declared more than once
keeps its **first** (lowest) line number, so a repeated name resolves to its
earliest occurrence. An axis with no authored line names yields an empty map.
