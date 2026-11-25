Reserved slider-scale step (`obscur_view::view::SLIDER_STEP`).

Reserved for the same reason as [`SLIDER_MIN`]. An absent step is a *free*
slider — `SliderScale::read` filters a non-positive or non-finite one back to
`None` — so omitting the attribute is a state and not a mistake.
