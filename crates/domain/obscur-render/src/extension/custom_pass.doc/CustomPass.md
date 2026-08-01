A descriptor of a privileged custom render pass a trusted extension runs
(spec §5.3).

Trusted, privileged extensions may inject backend-specific passes — for
example a GPU post-process or an overlay — scheduled by [`Self::stage`]
relative to the ordinary scene. A pass is pure data describing *what* to run
and *when*; it does **not** redefine ordinary Canvas semantics (spec §5.3).
A pass that uses restricted user shaders sets [`Self::requires_shaders`],
which gates a stricter capability check at grant time.
