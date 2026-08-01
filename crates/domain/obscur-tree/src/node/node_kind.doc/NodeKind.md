The classification of a retained tree node.

Every retained node carries exactly one `NodeKind` describing what it
represents in the tree. Text is a real node (spec §4.16), and
system-generated parts are modelled as [`NodeKind::Synthetic`] retained
nodes — hidden from the component tree but visible in DevTools.
