Collapses a single layer back to the flat one-value
[`PropertyValue`] it corresponds to.

This is how a one-element layer list stays **byte-identical** to the
historical single-value `background-image` result: the CSS lowering keeps
the flat variant for one layer and only emits
[`PropertyValue::BackgroundLayers`] for two or more.
