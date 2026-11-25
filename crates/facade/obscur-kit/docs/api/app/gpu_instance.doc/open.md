Create the instance, the surface and the adapter together, because on this API
they are not separable: an instance is a set of backends, a surface belongs to
an instance, and whether an adapter exists is only answerable once both are
built. Falling back from one backend set to another therefore means building all
three again, which is why the caller hands in a closure that makes the surface
rather than a surface.

Two attempts, both fixed — `attempts()` has the list and the reason it takes no
input. Nothing outside this file can shorten it, so there is no configuration in
which the program asks for no backend at all and dies before its first frame.

The error returned is the last one anything reported — a surface that could not
be created, or an adapter request that found nothing — because that is the one
describing the configuration the caller actually has.

**Not** a place to put backend preference logic beyond this. Which adapter is
chosen inside a backend set is `request_adapter`'s business, and the runners
report what it picked (`gpu_diagnostic`).
