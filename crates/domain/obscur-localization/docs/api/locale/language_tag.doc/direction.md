Returns the base writing direction this locale is written in
(spec §7.2 RTL integration, §7.3).

An explicit script decides it outright — that is what a script subtag is
*for*, and `az-Arab` is right-to-left while `az-Latn` is not. With no
script the language's likely script decides, via CLDR's likely-subtags
inference (see [`script_direction`](crate::locale::script_direction)); an
unknown language answers left-to-right.

This is the *base* direction only. It seeds a paragraph; the per-character
bidi levels inside one are the itemizer's job, and a Hebrew paragraph
still lays its embedded Latin out left-to-right.
