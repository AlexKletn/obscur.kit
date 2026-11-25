A single-value slider over the default 0..1 scale.

The argument is a VALUE, not a fraction. Widening the scale afterwards with
[`over`](Slider::over) does not reinterpret it — `Slider::new(0.7).over(0.0, 10.0)`
is still 0.7, near the bottom of its new range.
