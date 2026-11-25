The mutable bookkeeping threaded through a single validation pass
(spec §2.5).

A context tracks the two pieces of cross-node state a walk needs: the set of
node ids [`record_id`](ValidationContext::record_id) has already seen (so
duplicates can be detected), and the set of capability ids the host grants.
When the granted set is empty the validator does not enforce capabilities —
[`checks_capabilities`](ValidationContext::checks_capabilities) reports that.
