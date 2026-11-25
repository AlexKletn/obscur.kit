A headless menu of activatable items (spec §2.6, §6.7).

`Menu` owns its [`MenuItem`]s, whether the menu is currently
[`open`](Self::is_open), and which item (if any) is
[`highlighted`](Self::highlighted). It bakes in **no** visual theme:
[`view`](Self::view) derives a [`SemanticRole::Menu`] element carrying the
open/highlighted state as properties, with one
[`SemanticRole::MenuItem`]-shaped child element per item (each carrying its
disabled flag, a click event bound to the item's command, and a [`Text`]
label child), rendered by the style layer keyed on
[`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine covers open/close ([`open`](Self::open),
[`close`](Self::close) — which also clears the highlight —
[`toggle_open`](Self::toggle_open)), highlight movement
([`highlight`](Self::highlight), [`highlight_next`](Self::highlight_next),
[`highlight_prev`](Self::highlight_prev), which wrap and skip disabled
items), and committing a choice ([`activate`](Self::activate), which returns
the highlighted enabled item's command and closes the menu).
