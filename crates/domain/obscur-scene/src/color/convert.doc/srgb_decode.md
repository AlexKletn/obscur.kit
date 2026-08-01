Inverse sRGB transfer: decode a gamma-encoded sRGB channel to linear light
(CSS Color 4 §10.2, "lin_sRGB"). Input and output are nominally `0.0..=1.0`;
the sign is preserved for out-of-range inputs.
