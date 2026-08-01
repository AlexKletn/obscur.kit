Returns the tag as a four-character string slice.

The stored bytes are ASCII for every ISO-15924 code, so this never fails
for well-formed tags; for the constants defined here the conversion is
always valid.
