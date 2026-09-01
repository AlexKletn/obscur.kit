The control: the same scene, composed for the same 90px buffer, rasterized into
a destination one column and one row short — the finding's hypothesis, staged
deliberately.

It registers as 0.59 px of missing circle, well past the quarter pixel
`assert_is_a_circle` allows. Without this, the passing tests above would only
show that the check is quiet, not that it can speak.
