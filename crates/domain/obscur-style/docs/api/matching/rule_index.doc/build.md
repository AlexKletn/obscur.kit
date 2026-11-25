Files every rule of every sheet, in document order, into the buckets a
candidate walk reads.

Linear in the total number of rules and allocated once per sheet set. The
`sheets` handed here are the ones that must be handed to
[`RuleIndex::candidates`] later: the index stores positions, not rules, and
[`RuleIndex::describes`] is what checks the two agree.
