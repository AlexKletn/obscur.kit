The Linux appearance source, composed.

Holds all three readers and consults whichever
[`AppearanceSource`](super::AppearanceSource) names. The composition is the
substance: **merged, not first-wins.** A KDE desktop running a portal answers
the scheme twice and the palette once, and stopping at the first source that
said anything would throw away every colour on the machine.

Earlier sources outrank later ones field by field, so the portal — which agrees
with the rest of the desktop — decides the scheme, and Qt still supplies the
colours nothing else has.
