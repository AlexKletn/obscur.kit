A scratch color texture (`COLOR_MATRIX_FORMAT`, sampled + render target) the
pass writes its transformed output into, `width`×`height` clamped to
[`MAX_EXTENT`] per axis. The caller owns it so it can feed a following blur /
composite, keeping it alive until the encoder is submitted.
