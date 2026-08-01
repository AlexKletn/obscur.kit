Cached owned wgpu color-matrix pipeline for the CSS color-matrix `filter`
functions (grayscale/brightness/contrast/saturate/sepia/invert/hue-rotate/
opacity). Built lazily on the first color-filtered frame and reused; a scene
with no color-matrix layer never constructs it (byte-identical to before).
