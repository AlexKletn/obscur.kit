The pass multiplies colour by alpha, and writes nothing but the source.

Guards the reason this blit exists at all — straight alpha in, premultiplied out
— and, in its last assertion, the condition that makes `LoadOp::Clear` safe: the
target carries the source and only the source. A texel the triangle failed to
cover would come back as the clear colour instead.

It needs a GPU, so it is `#[ignore]`d — counted as ignored, never as a pass — and
fails outright if it is asked for where no adapter exists. See `gpu`.
