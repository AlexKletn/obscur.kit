§5.16 "- DPI variants;" and "- intrinsic size;", with §4.5 "- layout uses
logical units; - renderer converts to physical pixels; - fractional scale and
multiple DPI values are supported;".

An `auto`/`auto` replaced element takes its intrinsic size, and that size is
in logical units and does not vary with the window's device scale.

WEAKENED 2026-07-19, DPI premise only. The case asserted 200×100 for a
400×200 asset, on the premise that the `@2x` in the FILENAME `pic@2x.png`
declares a 2× density that halves the intrinsic logical size. §5.16 says only
"- DPI variants;" — one bare bullet. It does not say that a density is
carried in the filename, that `@Nx` is the suffix, that the suffix is parsed
off an explicitly authored `src`, or that doing so is the mechanism by which
DPI variants are selected at all (a manifest, a `srcset`-style declaration
and an explicit API are all equally consistent with the text). The expected
size is now the asset's own 400×200.

The auto-sizing half is DELIBERATELY LEFT INTACT and is expected to fail: the
measured box is (400, 0), so `height: auto` on a replaced element collapses
to zero and intrinsic aspect ratio is not applied. That is the registered
real gap in this case, and it is now the only thing standing between it and
green.

Candidate spec amendment: name the mechanism behind "- DPI variants;" — how a
density-annotated asset is declared and how its intrinsic logical size is
derived from its pixel size. Until then no test can distinguish a runtime
that supports DPI variants from one that has never heard of them.
