How sensitive a piece of UI or data is (spec §6.17, §7.12).

Ordered least → most sensitive so the derived [`Ord`] ranks sensitivity:
`Public < Sensitive < CaptureProtected < Secret`. The predicates
[`redact_in_diagnostics`](Self::redact_in_diagnostics) and
[`exclude_from_capture`](Self::exclude_from_capture) are monotone in that
order.
