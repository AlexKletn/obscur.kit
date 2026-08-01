Returns whether a change of this kind requires a relayout pass.

True for [`Text`](InvalidationKind::Text),
[`Style`](InvalidationKind::Style) (layout style is derived from computed
style, so a style change feeds layout), [`Layout`](InvalidationKind::Layout),
[`Tree`](InvalidationKind::Tree) and
[`Environment`](InvalidationKind::Environment).
