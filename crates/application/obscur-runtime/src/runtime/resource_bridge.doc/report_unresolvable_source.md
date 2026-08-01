Reports an image source that cannot resolve because no loader/decoder is
installed (spec §5.15 "The kit does not perform arbitrary network access by
itself.").

The dedup set makes this a *state*, not a per-frame log flood: a URL
referenced across many frames is named exactly once. There is no
[`ResourceId`](obscur_identity::ResourceId) to attach — nothing was
requested — so the source is named in the message instead.
