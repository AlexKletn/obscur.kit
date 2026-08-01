Host-registered font faces keyed by the run font handle (spec §5.2). A run
tagged with handle `run.font()` is rasterized against the matching sfnt
bytes; when no font is registered for the handle the run falls back to
box-coverage cells. The bytes are shared (`Arc`) so `clone` is cheap and a
borrowed [`FontRef`] is rebuilt per draw (bytes must outlive the borrow).
