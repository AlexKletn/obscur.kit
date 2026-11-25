Reserved progress value (`obscur_widgets::Progress::PROP_VALUE`).

The fraction a determinate bar has filled, in `0..=1`. Reserved rather than
hashed so that an authored `<progress value="0.5"/>` carries the same property
the Rust builder's own `Progress::new(0.5).view()` carries — one node shape from
two spellings, which is what lets the widget's painter and its factory read one
symbol instead of two.
