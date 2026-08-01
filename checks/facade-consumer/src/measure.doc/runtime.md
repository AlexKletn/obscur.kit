A headless runtime of `width`×`height`, the one [`measure`] uses.

Exposed on its own because a host that registers components has to hold the
runtime itself: registration happens before the first render and outlives it
(§2.14). [`measure_in`] is then the same measurement against that runtime.
