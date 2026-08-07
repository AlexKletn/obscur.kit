Reserved slider-scale minimum (`obscur_view::view::SLIDER_MIN`).

An authored `<slider min="…">` binds here rather than to a hashed name because
the readers of a slider's scale live below every front end and cannot call this
module: `SliderScale::read` (the view tier) and the runtime's drag
(`SliderControl::enclosing`) both look this symbol up by value. A hashed `min`
would be carried through the whole tree and then read by nobody, so the control
would silently keep the default `0..1` scale.
