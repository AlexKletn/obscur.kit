The width `text-align` aligns `node`'s lines within: its containing
block's content width (CSS Text 3 §7.1).

**Not the text box's own width**, which is where this used to look and why
`text-align` did nothing at all. A text node is shrink-wrapped to its
longest line, so there is never any slack inside it to distribute — the
offset came out `0` however the property was set. CSS aligns line boxes
within the containing block, and that is the parent's content box.

Never narrower than the box itself, so a missing or smaller parent box
leaves the previous behaviour exactly in place. A percentage padding is
read as `0`: percentages resolve against the containing block during
layout, which this pass does not redo — every other unit has already been
folded to pixels by the computed-value pass.
