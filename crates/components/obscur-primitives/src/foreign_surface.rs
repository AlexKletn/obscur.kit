//! `ForeignSurface`: an embedded external-surface primitive (spec §2.6, §5.5).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("../docs/api/foreign_surface.doc/ForeignSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignSurface {
    surface: Box<str>,
}

impl ForeignSurface {
    #[doc = include_str!("../docs/api/foreign_surface.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0008);

    #[doc = include_str!("../docs/api/foreign_surface.doc/PROP_SURFACE.md")]
    pub const PROP_SURFACE: InternedSymbol = InternedSymbol::from_raw(0x00C1_0801);

    /// Creates a foreign surface referencing the opaque handle `surface`.
    #[must_use]
    pub fn new(surface: impl Into<Box<str>>) -> Self {
        Self {
            surface: surface.into(),
        }
    }

    /// Returns the opaque surface handle key.
    #[must_use]
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// Lowers this surface into its [`ViewNode::Element`], emitting the surface
    /// handle as a property.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID).with_prop(ViewProperty::new(
            Self::PROP_SURFACE,
            ViewValue::Text(self.surface),
        ))
    }

    /// Lowers this surface into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_surface() {
        assert_eq!(ForeignSurface::new("gpu-0").surface(), "gpu-0");
    }

    #[test]
    fn emits_surface_prop() {
        let node = ForeignSurface::new("gpu-0").into_view_node();
        if let ViewNode::Element {
            component, props, ..
        } = node
        {
            assert_eq!(component, ForeignSurface::COMPONENT_ID);
            assert_eq!(props[0].name(), ForeignSurface::PROP_SURFACE);
            assert_eq!(props[0].value(), &ViewValue::Text("gpu-0".into()));
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = ForeignSurface::new("s").view();
        assert_eq!(view.root().len(), 1);
    }
}
