#![doc = include_str!("docking.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_layout::algorithm::DockSide;
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("docking.doc/DockPanel.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockPanel {
    id: Box<str>,
    title: Box<str>,
    side: DockSide,
}

impl DockPanel {
    #[doc = include_str!("docking.doc/new.md")]
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, title: impl Into<Box<str>>, side: DockSide) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            side,
        }
    }

    #[doc = include_str!("docking.doc/id.md")]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[doc = include_str!("docking.doc/title.md")]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[doc = include_str!("docking.doc/side.md")]
    #[must_use]
    pub const fn side(&self) -> DockSide {
        self.side
    }
}

#[doc = include_str!("docking.doc/Docking.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Docking {
    panels: Vec<DockPanel>,
}

impl Docking {
    #[doc = include_str!("docking.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000B);

    #[doc = include_str!("docking.doc/DOCK_PANEL_ID.md")]
    const DOCK_PANEL_ID: ComponentId = ComponentId::from_raw(0x00D0_001B);

    #[doc = include_str!("docking.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B00);
    #[doc = include_str!("docking.doc/PROP_DOCK_SIDE.md")]
    const PROP_DOCK_SIDE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B01);
    #[doc = include_str!("docking.doc/PROP_TITLE.md")]
    const PROP_TITLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0B02);

    #[doc = include_str!("docking.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }

    #[doc = include_str!("docking.doc/panel.md")]
    #[must_use]
    pub fn panel(mut self, panel: DockPanel) -> Self {
        self.panels.push(panel);
        self
    }

    #[doc = include_str!("docking.doc/with_panels.md")]
    #[must_use]
    pub fn with_panels<I>(mut self, panels: I) -> Self
    where
        I: IntoIterator<Item = DockPanel>,
    {
        self.panels.extend(panels);
        self
    }

    #[doc = include_str!("docking.doc/add_panel.md")]
    pub fn add_panel(&mut self, panel: DockPanel) {
        self.panels.push(panel);
    }

    #[doc = include_str!("docking.doc/remove_panel.md")]
    pub fn remove_panel(&mut self, id: &str) -> bool {
        if let Some(index) = self.panels.iter().position(|p| &*p.id == id) {
            self.panels.remove(index);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("docking.doc/dock.md")]
    pub fn dock(&mut self, id: &str, side: DockSide) -> bool {
        if let Some(panel) = self.panels.iter_mut().find(|p| &*p.id == id) {
            panel.side = side;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("docking.doc/panel_by_id.md")]
    #[must_use]
    pub fn panel_by_id(&self, id: &str) -> Option<&DockPanel> {
        self.panels.iter().find(|p| &*p.id == id)
    }

    #[doc = include_str!("docking.doc/panels.md")]
    #[must_use]
    pub fn panels(&self) -> &[DockPanel] {
        &self.panels
    }

    #[doc = include_str!("docking.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    #[doc = include_str!("docking.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    #[doc = include_str!("docking.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Group
    }

    #[doc = include_str!("docking.doc/side_name.md")]
    const fn side_name(side: DockSide) -> &'static str {
        match side {
            DockSide::Top => "top",
            DockSide::Bottom => "bottom",
            DockSide::Left => "left",
            DockSide::Right => "right",
            DockSide::Fill => "fill",
        }
    }

    #[doc = include_str!("docking.doc/view.md")]
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
