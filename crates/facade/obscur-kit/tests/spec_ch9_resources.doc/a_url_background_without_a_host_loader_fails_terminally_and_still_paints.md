§5.15 Resource Hub — "Sources:" / "- URL through user/host loader;" and "The
kit does not perform arbitrary network access by itself."

NG-URLIMG-01.

WEAKENED 2026-07-19: this asserted the handle reaches the TERMINAL `failed`
state with no host loader registered, and measured `Pending`. §5.15 lists
"- loading/ready/failed states;" among the hub's Features and "- URL through
user/host loader;" among its Sources, and closes with "The kit does not
perform arbitrary network access by itself." That the three states exist is
not a statement about which one an unserviceable request lands in, and the
text nowhere says the hub must synthesise a terminal failure for a source it
has no loader for — parking until a loader is installed is at least as
defensible, and is what a hot-reloaded or late-bound loader would want.

What is kept, and strengthened, is the sentence that IS normative: with no
loader, nothing fetched anything, so the handle must not be `Ready`. An
implementation that quietly performed an HTTP GET now fails here. The
frame-still-paints and reached-the-hub assertions are unchanged.

Candidate spec amendment: state the terminal behaviour of a request whose
source has no registered loader — `failed` immediately, or `loading` until
one appears — and if the latter, whether a diagnostic is emitted, because a
handle parked in `loading` forever is indistinguishable from a slow network
to every caller of `state()`.
