Canonical CSS Grid track-sizing algorithm (drafts.csswg.org/css-grid-2 §11).

Every track carries TWO numbers — a growing floor `base` and a ceiling
`limit` (its "growth limit", possibly ∞) — plus its resolved min/max sizing
functions. The six staged steps below (§11.4 init, §11.5 resolve-intrinsic
span distribution, §11.5.1 distribute-up-to-limits, §11.6 maximise, §11.7
find-size-of-fr with restart, §11.8 stretch) replace the previous one-value
`(base, fr, floor)` single-pass model. Reimplemented from the spec; the
`taffy` crate was read for cross-reference only (not copied).

# Content contributions, and what `auto` approximates

§11.5 distributes an item's contribution to the tracks it spans, and WHICH
contribution depends on the sizing function of each spanned track. Three are
modelled here, one per intrinsic track type ([`Contributions`]):

* a `min-content` track takes the item's min-content contribution — measured
  under an offer of no space;
* a `max-content` track takes its max-content contribution — measured under an
  unbounded offer;
* an `auto` track takes a single measure of the item under the container's own
  loose offer. This is an APPROXIMATION, and the one place this file departs
  from the spec: `auto` is formally `minmax(min-content, max-content)`, whose
  base should grow to the min-content contribution and whose growth limit to
  the max-content one. The single loose measure lands between the two — it is
  what the item wants in the space this container has — and it is what every
  `auto` track in this engine has always been sized by. Splitting it is a
  behaviour change to the overwhelmingly common track type, so it stays a
  documented approximation; the two keyword tracks, which have no legacy
  behaviour to preserve, probe for real.

# Where this sits in the vendored spec

The section numbers throughout this file are the ones the algorithm was
implemented against (§11.4 init … §11.8 stretch). The vendored snapshot
`specs/w3c/css-grid-2.txt` numbers the same steps §12.4 … §12.8 under
§12.3 *Track Sizing Algorithm*, subgrid having taken §9 in the meantime;
the few `§12.x` citations below are that later numbering. The steps
themselves are the same steps.
