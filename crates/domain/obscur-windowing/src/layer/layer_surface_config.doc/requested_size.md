The size the surface should *request*, with a zero on every axis the
anchoring stretches — the encoding a compositor reads as "you decide".

Asking for a width while anchored to both side edges is not an error and
not honoured; normalising it here means the request sent on the wire says
what will actually happen, so a surface that later disagrees with its
configure event has a real reason to.
