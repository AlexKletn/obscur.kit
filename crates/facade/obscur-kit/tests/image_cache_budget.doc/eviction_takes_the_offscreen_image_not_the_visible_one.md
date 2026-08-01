Eviction drops the image **nobody can see** rather than the one the frame is
showing, even when the off-screen image was looked up more recently (spec §5.19
visibility priority).

The recency order is forced, not incidental: the resolve pass walks the tree in
document order, so `b.png` — placed 5000px down, far outside the 64px viewport
— is touched *after* the on-screen `a.png` every frame and is therefore always
the more recent of the two. A cache that ordered victims by recency alone would
evict the visible `a.png`; only reading [`VisibilityPriority`] saves it.
