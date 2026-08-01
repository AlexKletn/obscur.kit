End-to-end: **`text-align` moves the text** (CSS Text 3 §7.1, spec §5.10).

It did not, for a reason worth stating: a text node's box is shrink-wrapped to
its longest line, and the alignment offset was computed inside *that* box —
where there is no slack to distribute, so the offset was always `0` however
the property was set. The property parsed, cascaded and computed the whole
way; only the width it aligned against was wrong. CSS aligns line boxes
within the **containing block**, which is the parent's content box.

Each test authors real CSS, runs one real frame, and reads the *published*
display list, so anything from the CSS compiler to the scene coordinator
failing shows up here.
