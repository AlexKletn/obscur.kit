The `arrange` results retained across frames — the other half of §4.2's
"may cache measure/layout results", and what makes step 8's *arrange*
incremental rather than only its measure: a subtree the frame's
[`LayoutInvalidation`] cannot have touched, and whose parent put back at the
same origin under the same constraints, is served from its retained absolute
boxes instead of re-running its layout algorithms.

It is a matched pair with [`measure_cache`](Self::measure_cache) under the
same preconditions: every site that clears one clears this too, because a
changed size is a changed position.
