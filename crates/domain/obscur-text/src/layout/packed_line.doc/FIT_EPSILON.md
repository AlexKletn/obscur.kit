How much wider than the available width still counts as fitting, relatively.

Not a typographic allowance — it is smaller than any distance a renderer can
draw. It exists because the two numbers being compared are **the same glyph
advances added up in a different order**, and `f32` addition is not associative.

Layout measures a run once and sizes the text box from that total; paint then
re-wraps against that very box, rebuilding the total word by word. Measured on
the case that produced this constant: the box was `293.9999` and the running sum
reached `294.00003` — a disagreement of 1.3 × 10⁻⁴ px, which pushed the last
word of a sentence onto a second line (audit 2-D).

`1e-5` relative is roughly eighty times the error observed and still under a
tenth of a pixel at ten thousand pixels wide. Relative rather than absolute
because the error grows with the magnitude being summed.
