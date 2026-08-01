Returns the substring of the source text covered by `span`.

The span offsets are clamped to the text's byte length and then snapped
down to the nearest UTF-8 character boundaries, so this never panics even
for out-of-bounds spans or spans that split a multi-byte character. An
out-of-range or fully-collapsed span yields an empty string.
