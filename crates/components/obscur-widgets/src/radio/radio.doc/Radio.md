A headless two-state radio-button control (spec §2.6).

`Radio` mirrors [`Checkbox`](crate::checkbox::Checkbox) as a *two-state*
(selected / not) control: it owns its `selected` flag, a `disabled` flag,
the `label`, and the `command` it fires on change. It bakes in **no** visual
theme — [`view`](Self::view) derives a [`SemanticRole::RadioButton`] element
carrying its state as properties plus (as children) a circular `.radio-dot`
indicator sub-part — whose `.radio-fill` dot appears only when selected —
followed by the label [`Text`].

Grouping / single-selection enforcement is the caller's concern (a radio
group owns which member is selected); this control just reports and toggles
its own membership via [`select`](Self::select) / [`toggle`](Self::toggle).
