The clip's axis-aligned bounding rectangle.

For a shape clip this is the shape's bounds (see [`ClipShape::bounds`]).
For a mask clip this is `None`: a mask's extent depends on its source
content, which is not known to the clip itself. Never panics.
