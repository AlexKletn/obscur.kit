Whether this toolkit can actually load `path`'s bytes as a single face.

**TrueType/OpenType collections (`.ttc`) are excluded**, and this is a real
exclusion rather than caution: the shaping stack parses faces with
`read_fonts::FontRef::new`, which rejects a collection outright
(`InvalidSfnt(0x74746366)` — the `ttcf` tag) because a collection needs a face
index this format string never carries. A `.ttc` handed onward would register
as a face with no readable metrics and no `cmap` coverage, so it would resolve
nothing and its text would silently *vanish*. Dropping it here instead lets the
caller fall through to a face that works.

The cost is real and worth naming: on a host whose only face for a script is a
collection — CJK installations commonly ship `.ttc` — this reports no face at
all rather than a broken one. Collection support is unbuilt.
