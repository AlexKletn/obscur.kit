The scroll container `c`'s stored offset, re-clamped to `c`'s **live**
range — so a layout change that shrinks the content never surfaces a stale
over-scroll.
