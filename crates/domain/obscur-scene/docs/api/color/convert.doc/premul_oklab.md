An sRGB colour as alpha-premultiplied Oklab.

Split out of the interpolation so a gradient pair pays for it twice, not thirty
times: three `powf(2.4)` and three `cbrt` per call, and the endpoints do not
change across the samples drawn between them.
