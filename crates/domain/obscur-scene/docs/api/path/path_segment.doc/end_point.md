Returns the point this segment moves the pen to, if any.

For [`PathSegment::Close`] there is no explicit end point (it returns to
the subpath start), so this yields `None`.
