The single width and color of a node's border when it is **uniform** and
every side is **dashed** (or `dotted`, which folds to dashed), else `None`.

Mirrors [`uniform_border`] but gates on the `dashed` style: all four sides
share one positive pixel width, one color, and the `dashed` style. Only such a
border paints as a single dashed stroke; a non-uniform dashed border (mixed
widths / colors, or a mix with solid / no sides) returns `None` and falls back
to [`border_fills`] (which paints only the `solid` sides — the documented
non-uniform fallback, so mixed dashed sides simply do not paint).

Like [`uniform_border`], the four widths arrive already read as
[`BorderWidths`] — the second of the three attempts must not re-read what the
first one already looked up.
