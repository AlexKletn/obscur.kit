A headless selectable list (spec §2.6, §4.9).

`List` owns its `items` (display labels), the set of currently
[`selected`](Self::selected_indices) item indices, and a
[`multi`](Self::is_multi) flag choosing between single- and multi-selection.
It bakes in **no** visual theme: [`view`](Self::view) derives a
[`SemanticRole::List`] element carrying the multi-select flag, with one
[`SemanticRole::ListItem`]-shaped child element per item (each carrying its
selected flag and a [`Text`] label child), rendered by the style layer keyed
on [`COMPONENT_ID`](Self::COMPONENT_ID).

The selection state machine covers [`select`](Self::select) (in single-select
mode this replaces the selection; in multi-select mode it adds to it),
[`toggle`](Self::toggle), [`deselect`](Self::deselect), and
[`clear_selection`](Self::clear_selection). Selection is stored by item index,
so it is stable and deterministic independent of any node identity.
