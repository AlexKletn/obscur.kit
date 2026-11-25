Returns the ordered multi-layer `background-image` list if this value is a
[`PropertyValue::BackgroundLayers`] (two or more layers, in author order).

A single-layer background is *not* this variant (it stays a flat
gradient/url value), so callers that want to treat both cases uniformly
must also check the single accessors.
