Blurs a single already-rendered [`BlurLayer`] sub-scene texture and
composites it onto `target`.

`source` is the sub-scene rendered into a margin-padded offscreen texture
whose top-left corresponds to absolute pixel `source_origin`; `layer` gives
the blur radius; `target` is the final frame the blurred result blends onto
(with `target_size` its extent). The work is recorded into `encoder`; the
caller submits it after the Vello render-to-texture passes so ordering holds.
