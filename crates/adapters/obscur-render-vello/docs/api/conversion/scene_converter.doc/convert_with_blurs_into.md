Converts into a scene the caller keeps, returning the blur layers.

The same as [`convert_with_blurs`](Self::convert_with_blurs) except that the
scene's buffers survive from one frame to the next. A `vello::Scene` is six
encoding streams; building a new one each frame regrows all six from zero for a
display list whose size barely changes between frames.
