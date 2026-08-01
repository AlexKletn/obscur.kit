§2.15: "- versioned;" and "- validated;"

WEAKENED 2026-07-19: this originally required the rejection's message to
contain the word "version". §2.15 says only "- versioned;" and "- validated;"
and does not describe a document that omits its version — nor is such a
document constructible in this codec, whose header is magic(4), format u16(2)
and IR version(4) at fixed positions, so deleting the version bytes truncates
the stream rather than producing a version-less one. The load correctly fails
("unexpected end of input"). What is kept is the assertion that carries the
requirement: the decoder does not assume a version and load anyway. Candidate
spec amendment: state whether a version is structurally mandatory in the IR
encoding (making this case unconstructible by design) and, if a version-less
document is representable at all, what diagnostic it must produce.
