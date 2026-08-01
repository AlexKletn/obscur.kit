**The keystone.** A line box must be sized by the installed face's own `hhea`
metrics, not by the idealized ratios.

This is the assertion the audit's 5-B(c) is about: shaping real glyphs into
ratio-sized boxes is real text in fake boxes, and it looks identical to working
until you measure. 21.792 ≠ 16.0 is the whole difference, and it can only come
from Noto Sans's tables — no arithmetic over the font size produces it.
