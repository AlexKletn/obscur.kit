Both ends, snapped to the scale and ordered.

Snapped on the way OUT and not as the builder runs: a builder that quantized
eagerly would destroy the author's value before the scale it belongs to had been
stated. `Slider::new(6.4).over(0.0, 10.0)` would have clamped 6.4 against the
default 0..1 and answered 1 — and the order the builder was called in would
have been something a caller had to know.
