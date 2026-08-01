Returns this image tagged with the asset density `scale` — how many of its
pixels span one logical CSS pixel (spec §5.16 DPI variants).

An `@2x` variant decodes to twice the pixel dimensions of the logical box
it stands in for, so tagging it `2.0` is what lets
[`logical_size`](Self::logical_size) report the size it should *lay out*
at rather than its raw pixel count. A non-finite or non-positive `scale`
is ignored in favour of the base `1.0`, since it could only produce a
degenerate logical size.
