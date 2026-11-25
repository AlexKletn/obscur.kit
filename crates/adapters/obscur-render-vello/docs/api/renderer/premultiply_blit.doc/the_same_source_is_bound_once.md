Binding the same source view twice keeps the same bind group.

The paired half of `a_second_source_is_what_the_second_blit_draws`: that one
fails if the cache never misses, this one fails if it never hits. Without it the
cache could be "fixed" by deleting it, and the patch would evaporate with the
test still green.

It needs a GPU, so it is `#[ignore]`d — counted as ignored, never as a pass — and
fails outright if it is asked for where no adapter exists. See `gpu`.
