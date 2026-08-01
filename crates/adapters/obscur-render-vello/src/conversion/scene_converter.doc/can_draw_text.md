Whether a run tagged with handle `id` will draw at all here, or be skipped
for want of a font (spec §5.2).

[`font_for`](Self::font_for)'s own rule, exposed so the renderer can tell a
host *which* runs its scene will lose. It has to be that same rule: a scan
that disagrees with the draw path reports a skip that did not happen, or
stays quiet about one that did.
