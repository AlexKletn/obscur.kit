Whether fontconfig is usable here — i.e. both tools ran.

A host with no fontconfig (a bare container, a non-Linux target, a
stripped image) is a normal state, not an error: the caller is expected to
supply bytes some other way.
