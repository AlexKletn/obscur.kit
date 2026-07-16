# Vendored test fonts

These two faces exist so the toolkit's **rendering assertions are hermetic**. A test
that reads whatever font the developer's machine happens to have installed asserts
against a different face on every machine, and advances/line heights drift with it.
Everything here is checked in, so a line box measured in CI is the line box measured
locally.

| File | Family | upem | `0` advance | hhea asc/desc/gap |
|---|---|---|---|---|
| `NotoSans-Regular.ttf` | Noto Sans (proportional) | 1000 | 572 | 1069 / −293 / 0 |
| `NotoSansMono-Regular.ttf` | Noto Sans Mono (fixed-pitch) | 1000 | 600 | 1069 / −293 / 0 |

The mono face is not decoration: `ch` resolves to the `0` glyph's advance, and the
`0.5em` substitute CSS Values 4 §6.1.2 permits when that advance is unmeasurable is
**8.0px at a 16px font-size against this face's real 9.6px** — a 20% error on the one
unit that exists to size a column by character count. A fixed-pitch face is what makes
that error visible to a test.

## Provenance

Copied byte-for-byte from Arch Linux's `noto-fonts` 1:2026.07.01-1
(`/usr/share/fonts/noto/`). Verify with:

```
sha256sum -c SHA256SUMS
```

Deliberately **not** taken from `/usr/share/fonts/WindowsFonts/` — that directory
mixes provenance, and `fc-match sans-serif` on the authoring machine resolves there,
which is precisely why these are vendored rather than enumerated.

## Licence

**SIL Open Font License 1.1** — see `LICENSE`.

Read from each font's own `name` table (nameID 13, "License Description"), *not* from
the packager's `/usr/share/licenses/noto-fonts/LICENSE`: that file is an unfilled
Apache-2.0 template (its notice still reads `Copyright [yyyy] [name of copyright
owner]`) and does **not** describe these binaries. The copyright line in `LICENSE` is
the fonts' own nameID 0. The licence body is the canonical OFL 1.1 text.

Neither face declares a Reserved Font Name, so OFL §3 imposes no renaming obligation
here. OFL §2 does require this notice travel with the binaries — keep `LICENSE` beside
them.

## Coverage, and what these fonts cannot test

Both faces cover Latin, Greek and **Cyrillic**. Neither covers Arabic or Hebrew — they
carry **no RTL script at all**. So they can exercise bidi *itemization* (run splitting
by resolved bidi level, which does not need coverage) but they **cannot** prove RTL
*shaping* or reordering of real glyphs. An RTL face would have to be vendored for that;
none is.
