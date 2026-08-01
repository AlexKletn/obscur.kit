Contains `diagnostic` — a failure that escaped a descendant's disabled
boundary — at this boundary, returning the failure as recorded here and
the fallback UI that replaces *this* component's subtree.

The retry budget belongs to the boundary the failure happened in, so a
boundary catching an escaped failure replaces its subtree rather than
retrying: the replacement unmounts the descendant that failed, which is
what containing it means.
