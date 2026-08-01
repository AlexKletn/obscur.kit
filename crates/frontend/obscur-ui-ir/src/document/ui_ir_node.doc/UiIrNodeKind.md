The category of a [`UiIrNode`] (spec §2.5).

A node's kind fixes how it is interpreted: a raw element, a text leaf, a
component instance, a transparent fragment, or the control-flow
[`Conditional`](UiIrNodeKind::Conditional)/[`Loop`](UiIrNodeKind::Loop)
forms. Every kind except [`Text`](UiIrNodeKind::Text) can hold children;
[`is_container`](UiIrNodeKind::is_container) reports that.
