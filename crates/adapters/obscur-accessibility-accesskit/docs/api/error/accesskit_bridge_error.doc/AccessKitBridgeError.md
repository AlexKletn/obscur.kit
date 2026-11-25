A failure raised while converting the domain semantic tree into AccessKit
types (spec §7.1, §2.7).

This is the adapter's own diagnostic, surfaced instead of a panic when a
conversion cannot faithfully produce an [`accesskit::TreeUpdate`]. The
determinism boundary of spec §7.1 requires the tree structure, roles, and
states to map faithfully; when that is impossible the adapter reports the
concrete cause here.
