How the bytes of a foreign pixel buffer are laid out.

The list is short on purpose: these are the layouts a camera, a video decoder,
a screen capture or a software rasterizer actually hand over. Anything more
exotic — planar YUV, 10-bit, floating point — is the producer's job to convert
before it gets here, because doing it well needs colour information this type
does not carry.
