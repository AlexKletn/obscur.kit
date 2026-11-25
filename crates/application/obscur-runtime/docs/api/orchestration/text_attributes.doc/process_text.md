Runs `content` through `text-transform` (§2.1) and then `white-space`
collapsing (§4.1.1), returning the fully processed pre-shape string.

The two pre-shape rewrites compose in spec order: casing first (producing the
*transformed text*), then whitespace processing. When both are no-ops the
original `content` is returned borrowed (zero-copy, byte-for-byte identical).
