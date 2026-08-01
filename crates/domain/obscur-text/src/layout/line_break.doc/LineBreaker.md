A replaceable **line-break-opportunity port** (spec §5.10, UAX #14).

This is the seam through which a full Unicode line-breaking (UAX #14) or CJK
adapter can supply break opportunities without changing the wrapping engine —
the exact analogue of the [`Hyphenator`](super::Hyphenator) port for interior
hyphenation. The domain wraps text through a `&dyn LineBreaker`; the default
implementation ([`DefaultLineBreaker`]) reproduces the historical
whitespace-and-forced-break behaviour plus basic inter-ideograph CJK breaks,
while a host may install a tailored adapter behind the xtask boundary.

# Contract

- The returned breaks are byte offsets into `text` (UTF-8), each marking a
  position at which the following text may (soft) or must (hard) start a new
  line, in **ascending** offset order.
- The result must be **deterministic**: the same `text` and `white_space`
  always yield the same opportunities, so the measure and compose passes agree
  on the same lines.

It is deliberately **object-safe** so the runtime can hold a
`Box<dyn LineBreaker>` / pass a `&dyn LineBreaker`.
