§3.5 Rules — "animation progress is sampled from time/value, never slowed to
match missed frames".

A 1000 ms 0→100 animation stalled from t=0 to t=900 with no frames presented
samples **90** in the next frame — not 1.67 (one 16 ms step) and not 100.
