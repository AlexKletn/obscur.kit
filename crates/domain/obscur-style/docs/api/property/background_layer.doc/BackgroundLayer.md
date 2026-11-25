A single `background-image` layer: a gradient or a `url(...)` reference.

Layers are stored in CSS author order (first = top). The scene coordinator
paints them back-to-front (last first) so the first-listed layer ends up on
top (spec §5.2).
