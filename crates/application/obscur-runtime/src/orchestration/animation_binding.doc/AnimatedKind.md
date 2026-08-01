How a sampled scalar [`MotionValue`](obscur_motion::value::MotionValue) is
lowered into a concrete [`PropertyValue`] for a computed-style override.

The motion engine speaks in typed [`MotionValue`]s; the target property
decides what that value *means* in CSS terms. `opacity` and `flex-grow` want
a bare [`Number`](PropertyValue::Number) sampled from a
[`Scalar`](MotionValue::Scalar); `width`/`height`/margins want a pixel
[`Length`](PropertyValue::Length), also from a scalar; `transform` wants a
[`Transform`](PropertyValue::Transform) affine sampled from a
[`Transform`](MotionValue::Transform) matrix — which interpolates by
decomposition, never component-wise (CSS Transforms §14). This enum is the
small, property-kind-driven bridge between the two.
