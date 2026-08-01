The default maximum per-side pixel dimension a decoded image may have (spec
§5.16 safety limits). 16384 comfortably exceeds real UI assets while bounding
the worst-case RGBA allocation to about one gigabyte.
