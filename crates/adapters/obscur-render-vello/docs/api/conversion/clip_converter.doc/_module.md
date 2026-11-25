Converts an obscur-scene [`Clip`] into a kurbo [`BezPath`] clip shape.

Pure, backend-local conversion (spec §2.7). Vello models a clip as a layer
bounded by a shape, so a clip becomes the [`BezPath`] outline it confines
drawing to. Geometry is preserved exactly (spec §5.2).
