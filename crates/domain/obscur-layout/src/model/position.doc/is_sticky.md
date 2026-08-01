Returns `true` only for [`Sticky`](Self::Sticky) — the one **in-flow**
scheme whose used position is scroll-clamped to the scrollport edge.

A sticky node is *not* [`is_relative`](Self::is_relative) (it is not
offset by a fixed inset delta) and is *not*
[`is_out_of_flow`](Self::is_out_of_flow) (it keeps its flow box and never
perturbs its siblings); the coordinator recognises it here and applies the
per-axis sticky clamp instead of the relative offset.
