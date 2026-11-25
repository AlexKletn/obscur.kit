A headless dock host arranging panels against edges (spec §2.6, docking
primitives).

`Docking` owns an ordered [`Vec`] of [`DockPanel`]s. It bakes in **no** visual
theme: [`view`](Self::view) derives a [`SemanticRole::Group`] element with one
[`SemanticRole::Region`]-shaped child per panel (keyed by the panel id, and
carrying its dock side and title), rendered by the style layer keyed on
[`COMPONENT_ID`](Self::COMPONENT_ID). The concrete edge placement is performed
by [`obscur_layout::algorithm`]'s dock layout from the panels' [`DockSide`]s.

The state machine covers [`add_panel`](Self::add_panel),
[`remove_panel`](Self::remove_panel), and re-docking an existing panel to a
new edge ([`dock`](Self::dock)).
