Validates every element in `doc`, recursively, reporting each declared
requirement that is not granted (spec §2.13).

For each element carrying a `requires="cap"` string attribute whose value
`cap` is not in the granted set, a diagnostic
`"required capability '<cap>' is not granted"` is produced at the
element's byte offset.
