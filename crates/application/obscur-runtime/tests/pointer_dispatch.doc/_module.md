Where a pointer lands, and what that costs the tree: hit-test → hover → focus →
pseudo-class → paint (spec §2.3, §4.14, phase 3 step 2).

Ten tests about one chain of consequences. A contact at a coordinate resolves to
the innermost node whose box contains it; that node and its ancestors take
`:hover`, a press adds `:active` and seats focus, and the release takes `:active`
away again. Everything after the hit-test is a *derived* fact, and every test
here asserts one link of that derivation rather than the whole of it.

Two of them close the loop by reading the display list instead of the pseudo
state: `.b:hover` and `.a:focus` are the only rules in
[`InteractionPanel::sheet`] that paint anything, so a fill that was absent from
the cold frame and present in the next one is proof that the interaction reached
the cascade — not merely that a bit was set on a node.

The rest are the boundaries. A pointer outside every box clears hover rather than
keeping the last one; a second move *within* the same node changes nothing and
must therefore schedule no frame at all; a hit-test before any frame has run has
an empty spatial index and must answer "nothing" instead of panicking; and a
control carrying the reserved `disabled` attribute is inert to focus and to
activation while still hovering, because CSS hovers a disabled element.

[`InteractionPanel::sheet`]: obscur_testkit::fixture::InteractionPanel::sheet
