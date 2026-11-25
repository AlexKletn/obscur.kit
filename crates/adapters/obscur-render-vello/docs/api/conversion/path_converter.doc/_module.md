Converts an obscur-scene [`Path`] into a kurbo [`BezPath`].

Pure, backend-local geometry conversion (spec §2.7). The domain's
backend-agnostic path segments are translated one-for-one into kurbo path
elements, preserving the scene geometry exactly (spec §5.2 — only
rasterization is Vello's; geometry is authoritative).
