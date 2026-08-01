Arms a long press for `pointer`. The caller decides whether the gesture is
bound at all — arming where it is not would spend half a second of frames
on every tap, almost always to conclude nothing.
