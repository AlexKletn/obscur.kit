The kind of physical device that produced a pointer event (spec §4.7).

Pointer routing is device-agnostic, but handlers sometimes need to adapt to
the source: whether it reports pressure, or whether it is a direct
(touch-the-target) rather than indirect (move-a-cursor) manipulation.
