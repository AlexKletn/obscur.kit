Advances every coasting container by `delta` seconds, scrolling each by
the distance its fling covered, and requests another animation frame while
any coast remains (spec §6.16, §4.17).

A coast that reaches the end of its content is **stopped** rather than
left decaying against the clamp: the list has arrived, and a fling that
went on being advanced would keep asking for frames while moving nothing.
