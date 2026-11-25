Returns whether this stage is backend-dependent, i.e. it can only be
produced by a concrete renderer (spec §7.4).

Every stage except [`Image`](Self::Image) is derived deterministically
from the retained tree with no GPU; the rendered image requires a
rasterizing backend.
