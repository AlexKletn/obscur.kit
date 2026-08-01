The scrollport rect (logical px). This implementation uses the **border
box** — matching the existing `overflow` paint clip — where CSS Overflow 3
defines the scrollport as the padding box: a documented simplification kept
deliberately in step with the coordinator's overflow clip so the clipped
region and the scrolled region coincide exactly.
