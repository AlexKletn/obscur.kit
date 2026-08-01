The viewport point a synthetic pointer aimed at `target` should land on,
or [`None`] when the target does not resolve to a laid-out node.

A coordinate is its own point. A node or `test-id` resolves to the centre
of that node's retained layout box, so the event lands where a user
clicking that element would put it — and then travels the real input path,
hit-test and all.
