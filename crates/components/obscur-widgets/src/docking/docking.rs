//! `Docking`: a headless dock host arranging panels against edges (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_layout::algorithm::DockSide;
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A single dockable panel in a [`Docking`] host.
///
/// Each panel carries a stable `id` (used for lookup and keyed reconciliation), a
/// display `title`, and the [`DockSide`] it is docked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockPanel {
    id: Box<str>,
    title: Box<str>,
    side: DockSide,
}

impl DockPanel {
    /// Creates a panel with stable `id`, display `title`, docked against `side`.
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, title: impl Into<Box<str>>, side: DockSide) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            side,
        }
    }

    /// Returns the panel's stable id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the panel title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the edge the panel is docked against.
    #[must_use]
    pub const fn side(&self) -> DockSide {
        self.side
    }
}

/// A headless dock host arranging panels against edges (spec §2.6, docking
/// primitives).
///
/// `Docking` owns an ordered [`Vec`] of [`DockPanel`]s. It bakes in **no** visual
/// theme: [`view`](Self::view) derives a [`SemanticRole::Group`] element with one
/// [`SemanticRole::Region`]-shaped child per panel (keyed by the panel id, and
/// carrying its dock side and title), rendered by the style layer keyed on
/// [`COMPONENT_ID`](Self::COMPONENT_ID). The concrete edge placement is performed
/// by [`obscur_layout::algorithm`]'s dock layout from the panels' [`DockSide`]s.
///
/// The state machine covers [`add_panel`](Self::add_panel),
/// [`remove_panel`](Self::remove_panel), and re-docking an existing panel to a
/// new edge ([`dock`](Self::dock)).
#[derive(Debug, Clone, PartialEq)]
pub struct Docking {
    panels: Vec<DockPanel>,
}

impl Docking {
    /// Stable component id for the docking widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000B);

    /// Stable component id for the per-panel child element.
    const DOCK_PANEL_ID: ComponentId = ComponentId::from_raw(0x00D0_001B);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 0B` (docking): role=`00`, dock-side=`01`,
    /// title=`02` (on panel children).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B00);
    /// Interned name of the per-panel `dock-side` property.
    const PROP_DOCK_SIDE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B01);
    /// Interned name of the per-panel `title` property.
    const PROP_TITLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B02);

    /// Creates an empty dock host with no panels.
    #[must_use]
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }

    /// Appends a single panel, returning the updated host (fluent builder).
    #[must_use]
    pub fn panel(mut self, panel: DockPanel) -> Self {
        self.panels.push(panel);
        self
    }

    /// Appends every panel from `panels`, returning the updated host.
    #[must_use]
    pub fn with_panels<I>(mut self, panels: I) -> Self
    where
        I: IntoIterator<Item = DockPanel>,
    {
        self.panels.extend(panels);
        self
    }

    /// Adds a panel to the host.
    pub fn add_panel(&mut self, panel: DockPanel) {
        self.panels.push(panel);
    }

    /// Removes the first panel whose id matches `id`, returning `true` when a
    /// panel was removed.
    pub fn remove_panel(&mut self, id: &str) -> bool {
        if let Some(index) = self.panels.iter().position(|p| &*p.id == id) {
            self.panels.remove(index);
            true
        } else {
            false
        }
    }

    /// Re-docks the panel with id `id` to `side`, returning `true` on success.
    /// Returns `false` (leaving state untouched) when no panel has that id.
    pub fn dock(&mut self, id: &str, side: DockSide) -> bool {
        if let Some(panel) = self.panels.iter_mut().find(|p| &*p.id == id) {
            panel.side = side;
            true
        } else {
            false
        }
    }

    /// Returns the panel with id `id`, if any.
    #[must_use]
    pub fn panel_by_id(&self, id: &str) -> Option<&DockPanel> {
        self.panels.iter().find(|p| &*p.id == id)
    }

    /// Returns all panels in dock order.
    #[must_use]
    pub fn panels(&self) -> &[DockPanel] {
        &self.panels
    }

    /// Returns the number of panels.
    #[must_use]
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Returns whether the host has no panels.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Returns the accessible role of this host ([`SemanticRole::Group`]).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Group
    }

    /// Maps a [`DockSide`] to its lowercase view-property name.
    const fn side_name(side: DockSide) -> &'static str {
        match side {
            DockSide::Top => "top",
            DockSide::Bottom => "bottom",
            DockSide::Left => "left",
            DockSide::Right => "right",
            DockSide::Fill => "fill",
        }
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element with one
    /// region-role child per panel (keyed by the panel id, carrying its dock side
    /// and title), in dock order.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let children: Vec<ViewNode> = self
            .panels
            .iter()
            .map(|panel| {
                ViewNode::element(Self::DOCK_PANEL_ID)
                    .with_key(&*panel.id)
                    .with_prop(ViewProperty::new(
                        Self::PROP_ROLE,
                        ViewValue::from("region"),
                    ))
                    .with_prop(ViewProperty::new(
                        Self::PROP_DOCK_SIDE,
                        ViewValue::from(Self::side_name(panel.side)),
                    ))
                    .with_prop(ViewProperty::new(
                        Self::PROP_TITLE,
                        ViewValue::Text(panel.title.clone()),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(panel.title.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("group")))
            .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

impl Default for Docking {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Docking {
        Docking::new().with_panels([
            DockPanel::new("nav", "Navigator", DockSide::Left),
            DockPanel::new("out", "Output", DockSide::Bottom),
        ])
    }

    #[test]
    fn panel_accessors() {
        let p = DockPanel::new("id", "Title", DockSide::Right);
        assert_eq!(p.id(), "id");
        assert_eq!(p.title(), "Title");
        assert_eq!(p.side(), DockSide::Right);
    }

    #[test]
    fn add_and_len() {
        let mut d = Docking::new();
        d.add_panel(DockPanel::new("a", "A", DockSide::Top));
        assert_eq!(d.len(), 1);
        assert!(!d.is_empty());
    }

    #[test]
    fn remove_panel() {
        let mut d = sample();
        assert!(d.remove_panel("nav"));
        assert_eq!(d.len(), 1);
        assert!(!d.remove_panel("nav")); // unknown now
    }

    #[test]
    fn dock_changes_side() {
        let mut d = sample();
        assert!(d.dock("out", DockSide::Right));
        assert_eq!(
            d.panel_by_id("out").map(DockPanel::side),
            Some(DockSide::Right)
        );
        assert!(!d.dock("missing", DockSide::Top));
    }

    #[test]
    fn panel_lookup() {
        let d = sample();
        assert_eq!(
            d.panel_by_id("nav").map(DockPanel::title),
            Some("Navigator")
        );
        assert!(d.panel_by_id("nope").is_none());
    }

    #[test]
    fn semantic_role_is_group() {
        assert_eq!(sample().semantic_role(), SemanticRole::Group);
    }

    #[test]
    fn view_children_match_panels_and_side() {
        let d = sample();
        let view = d.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                children,
                ..
            } => {
                assert_eq!(*component, Docking::COMPONENT_ID);
                assert_eq!(children.len(), 2);
                match &children.nodes()[0] {
                    ViewNode::Element { props, .. } => {
                        let side = props
                            .iter()
                            .find(|p| p.name() == Docking::PROP_DOCK_SIDE)
                            .expect("dock-side prop present");
                        assert_eq!(*side.value(), ViewValue::from("left"));
                    }
                    ViewNode::Text { .. } => panic!("expected element child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
