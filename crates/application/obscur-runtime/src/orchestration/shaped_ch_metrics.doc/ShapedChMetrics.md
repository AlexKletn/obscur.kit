A [`ChMetrics`] port backed by a live [`ParagraphShaper`].

Borrows the shaper rather than owning one: the runtime already owns exactly
one (the scene coordinator's), and `ch` must be measured with *that* shaper —
a second one could disagree, which is the class of bug this port closes.

Not [`Debug`]: [`ParagraphShaper`] is not, which is precisely why `ChMetrics`
does not require it of an implementor.
