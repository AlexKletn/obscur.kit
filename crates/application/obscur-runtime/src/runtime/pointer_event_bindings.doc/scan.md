Recompute the set from a committed tree.

O(live nodes) with a two-comparison inner loop, and it stops as soon as all seven
are accounted for. It runs on structural frames only, which already walk the
tree several times over.
