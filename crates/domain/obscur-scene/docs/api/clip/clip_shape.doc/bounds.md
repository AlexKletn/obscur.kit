The shape's axis-aligned bounding rectangle.

For [`ClipShape::Rect`] and [`ClipShape::RoundedRect`] this is the
rectangle itself (rounding only removes area, so the outer rectangle
still bounds the shape). For [`ClipShape::Path`] this delegates to
[`Path::bounds`], which returns `None` for an empty path. Never panics.
