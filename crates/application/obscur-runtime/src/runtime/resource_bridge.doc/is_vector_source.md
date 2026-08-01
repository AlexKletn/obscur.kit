Whether `src` names a vector document (spec §5.17).

Decided by extension rather than by sniffing the bytes: an author writing
`.svg` means one, and trying to parse every bitmap as SVG to find out would
cost a failed parse per image and turn a clear "not an image" into a confusing
"invalid SVG".
