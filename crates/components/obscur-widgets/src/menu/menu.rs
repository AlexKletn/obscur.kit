//! `Menu`: a headless menu of activatable items (spec §2.6, §6.7).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

/// A single entry in a [`Menu`].
///
/// Each item carries a display `label`, the `command` it fires when activated,
/// and whether it is currently [`enabled`](Self::is_enabled). Disabled items are
/// still rendered but are skipped by keyboard highlight navigation and cannot be
/// activated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    label: Box<str>,
    command: InternedSymbol,
    enabled: bool,
}

impl MenuItem {
    /// Creates an enabled item showing `label` that fires `command` when
    /// activated.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            enabled: true,
        }
    }

    /// Marks the item disabled, returning it (fluent builder).
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Returns the item label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the command fired when the item is activated.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns whether the item is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// A headless menu of activatable items (spec §2.6, §6.7).
///
/// `Menu` owns its [`MenuItem`]s, whether the menu is currently
/// [`open`](Self::is_open), and which item (if any) is
/// [`highlighted`](Self::highlighted). It bakes in **no** visual theme:
/// [`view`](Self::view) derives a [`SemanticRole::Menu`] element carrying the
/// open/highlighted state as properties, with one
/// [`SemanticRole::MenuItem`]-shaped child element per item (each carrying its
/// disabled flag, a click event bound to the item's command, and a [`Text`]
/// label child), rendered by the style layer keyed on
/// [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The state machine covers open/close ([`open`](Self::open),
/// [`close`](Self::close) — which also clears the highlight —
/// [`toggle_open`](Self::toggle_open)), highlight movement
/// ([`highlight`](Self::highlight), [`highlight_next`](Self::highlight_next),
/// [`highlight_prev`](Self::highlight_prev), which wrap and skip disabled
/// items), and committing a choice ([`activate`](Self::activate), which returns
/// the highlighted enabled item's command and closes the menu).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    items: Vec<MenuItem>,
    open: bool,
    highlighted: Option<usize>,
}

impl Menu {
    /// Stable component id for the menu widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0005);

    /// Stable component id for the per-item child element.
    const MENU_ITEM_ID: ComponentId = ComponentId::from_raw(0x00D0_0015);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 05` (menu): role=`00`, open=`01`,
    /// highlighted=`02`, disabled=`03` (on item children).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0500);
    /// Interned name of the `open` state property.
    const PROP_OPEN: InternedSymbol = InternedSymbol::from_raw(0x00D1_0501);
    /// Interned name of the `highlighted` index property (`-1` = none).
    const PROP_HIGHLIGHTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0502);
    /// Interned name of the per-item `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0503);

    /// Creates a closed, empty menu with no highlighted item.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            open: false,
            highlighted: None,
        }
    }

    /// Appends a single item, returning the updated menu (fluent builder).
    #[must_use]
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Appends every item from `items`, returning the updated menu.
    #[must_use]
    pub fn with_items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        self.items.extend(items);
        self
    }

    /// Opens the menu.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Closes the menu, clearing any highlighted item.
    pub fn close(&mut self) {
        self.open = false;
        self.highlighted = None;
    }

    /// Toggles the menu open/closed. Closing clears the highlight.
    pub fn toggle_open(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open = true;
        }
    }

    /// Highlights the item at `index`, returning `true` when `index` is in
    /// range; otherwise leaves the highlight untouched and returns `false`.
    pub fn highlight(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.highlighted = Some(index);
            true
        } else {
            false
        }
    }

    /// Moves the highlight to the next enabled item, wrapping around. With no
    /// current highlight this selects the first enabled item. A no-op when no
    /// item is enabled.
    pub fn highlight_next(&mut self) {
        self.highlighted = self.scan_enabled(self.highlighted, true);
    }

    /// Moves the highlight to the previous enabled item, wrapping around. With
    /// no current highlight this selects the last enabled item. A no-op when no
    /// item is enabled.
    pub fn highlight_prev(&mut self) {
        self.highlighted = self.scan_enabled(self.highlighted, false);
    }

    /// Scans for the next/previous enabled item relative to `from`, wrapping.
    /// With `from == None`, starts so the first (forward) or last (backward)
    /// enabled item is chosen. Returns the current highlight when no item is
    /// enabled.
    fn scan_enabled(&self, from: Option<usize>, forward: bool) -> Option<usize> {
        let len = self.items.len();
        if len == 0 {
            return None;
        }
        for step in 1..=len {
            let index = if forward {
                let base = from.unwrap_or(len - 1);
                (base + step) % len
            } else {
                let base = from.unwrap_or(0);
                (base + len - (step % len)) % len
            };
            if self.items[index].enabled {
                return Some(index);
            }
        }
        from
    }

    /// Activates the highlighted item, returning its command when an item is
    /// highlighted **and** enabled, then closes the menu. Returns `None` (and
    /// still closes) when nothing is highlighted or the highlighted item is
    /// disabled.
    pub fn activate(&mut self) -> Option<InternedSymbol> {
        let command = self
            .highlighted
            .and_then(|i| self.items.get(i))
            .filter(|item| item.enabled)
            .map(MenuItem::command);
        self.close();
        command
    }

    /// Returns whether the menu is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the highlighted item index, if any.
    #[must_use]
    pub const fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// Returns the number of items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the menu has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::Menu`]).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Menu
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role,
    /// open state, and highlighted index (`-1` when none), with one
    /// menuitem-role child element per item (each carrying its disabled flag, a
    /// click event bound to the item's command, and a [`Text`] label child).
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let highlighted = self
            .highlighted
            .map_or(-1, |i| i64::try_from(i).unwrap_or(-1));
        let children: Vec<ViewNode> = self
            .items
            .iter()
            .map(|item| {
                ViewNode::element(Self::MENU_ITEM_ID)
                    .with_prop(ViewProperty::new(
                        Self::PROP_ROLE,
                        ViewValue::from("menuitem"),
                    ))
                    .with_prop(ViewProperty::new(
                        Self::PROP_DISABLED,
                        ViewValue::Bool(!item.enabled),
                    ))
                    .with_event(ViewEventBinding::new(EVENT_CLICK, item.command))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(item.label.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("menu")))
            .with_prop(ViewProperty::new(
                Self::PROP_OPEN,
                ViewValue::Bool(self.open),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_HIGHLIGHTED,
                ViewValue::Int(highlighted),
            ))
            .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(raw: u32) -> InternedSymbol {
        InternedSymbol::from_raw(raw)
    }

    fn sample() -> Menu {
        Menu::new().with_items([
            MenuItem::new("Cut", cmd(0x10)),
            MenuItem::new("Copy", cmd(0x11)).disabled(),
            MenuItem::new("Paste", cmd(0x12)),
        ])
    }

    #[test]
    fn item_builder_and_valid_highlight() {
        let mut m = Menu::new().item(MenuItem::new("Only", cmd(0x1)));
        assert_eq!(m.len(), 1);
        assert!(m.highlight(0));
        assert_eq!(m.highlighted(), Some(0));
        assert!(!m.highlight(1));
    }

    #[test]
    fn highlight_next_wraps_and_skips_disabled() {
        let mut m = sample();
        m.highlight_next(); // None -> first enabled (0)
        assert_eq!(m.highlighted(), Some(0));
        m.highlight_next(); // skips disabled 1 -> 2
        assert_eq!(m.highlighted(), Some(2));
        m.highlight_next(); // wraps past disabled 1 -> 0
        assert_eq!(m.highlighted(), Some(0));
    }

    #[test]
    fn highlight_prev_wraps_and_skips_disabled() {
        let mut m = sample();
        m.highlight_prev(); // None -> last enabled (2)
        assert_eq!(m.highlighted(), Some(2));
        m.highlight_prev(); // skips disabled 1 -> 0
        assert_eq!(m.highlighted(), Some(0));
    }

    #[test]
    fn activate_returns_enabled_command_and_closes() {
        let mut m = sample();
        m.open();
        assert!(m.highlight(2));
        assert_eq!(m.activate(), Some(cmd(0x12)));
        assert!(!m.is_open());
        assert_eq!(m.highlighted(), None);
    }

    #[test]
    fn activate_disabled_highlight_is_none() {
        let mut m = sample();
        assert!(m.highlight(1)); // disabled item
        assert_eq!(m.activate(), None);
    }

    #[test]
    fn open_then_close_clears_highlight() {
        let mut m = sample();
        m.open();
        assert!(m.highlight(0));
        m.close();
        assert!(!m.is_open());
        assert_eq!(m.highlighted(), None);
    }

    #[test]
    fn semantic_role_is_menu() {
        assert_eq!(sample().semantic_role(), SemanticRole::Menu);
    }

    #[test]
    fn view_children_are_menuitems() {
        let m = sample();
        let view = m.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                children,
                ..
            } => {
                assert_eq!(*component, Menu::COMPONENT_ID);
                assert_eq!(children.len(), 3);
                for child in children.nodes() {
                    match child {
                        ViewNode::Element { props, .. } => {
                            let role = props
                                .iter()
                                .find(|p| p.name() == Menu::PROP_ROLE)
                                .expect("role prop present");
                            assert_eq!(*role.value(), ViewValue::from("menuitem"));
                        }
                        ViewNode::Text { .. } => panic!("expected element child"),
                    }
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
