The platform declined the request, e.g. to prevent focus stealing.

Per spec §6.3, focus-stealing restrictions are normal platform behavior,
**not** a toolkit failure: a `Denied` result is an expected outcome an
application should handle gracefully, not an error condition.
