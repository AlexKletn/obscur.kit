Inputs used to evaluate a [`RuleCondition`](crate::matching::RuleCondition).
Viewport and container dimensions use logical pixels.

An unset viewport dimension is non-restrictive. An unset container dimension
prevents a size query on that axis from matching. The runtime supplies the
latest arranged container sizes through [`with_query_containers`](Self::with_query_containers)
and attaches a selector subject through [`for_subject`](Self::for_subject).
[`with_container`](Self::with_container) supplies a single unnamed container
for standalone evaluation.
