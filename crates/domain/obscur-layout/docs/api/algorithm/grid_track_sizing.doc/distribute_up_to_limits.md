§11.5.1 core — *Distribute extra space up to limits*. Grows the affected
tracks in `tracks[lo..hi]` equally, freezing each as it reaches `limit_of`,
by repeatedly raising all still-growable tracks by the smallest per-track
headroom (or the even share, whichever is smaller). Accumulates into each
track's `increase`; returns the space that could not be placed (all affected
tracks hit their limit). Reused by §11.6 maximise.
