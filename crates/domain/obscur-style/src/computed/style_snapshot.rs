//! An immutable, versioned snapshot of a computed style: [`StyleSnapshot`]
//! (§1.6).
//!
//! A [`StyleSnapshot`] hands a node's [`ComputedStyle`] to heavy downstream
//! subsystems (layout, paint) together with a [`VersionStamp`], so a consumer can
//! cheaply tell whether the style it holds is stale — mirroring how the tree
//! hands out a versioned `TreeSnapshot`. The snapshot owns its style and exposes
//! read-only accessors; it has no mutators.

use crate::computed::{ComputedStyle, ComputedValue};
use crate::property::PropertyId;
use obscur_identity::VersionStamp;

/// An immutable, versioned view of a node's computed style.
///
/// Wraps an owned [`ComputedStyle`] plus the [`VersionStamp`] it was produced at.
/// It is read-only: consumers observe the style and compare version stamps but
/// never mutate the snapshot.
///
/// Because [`ComputedStyle`] may carry `f32` data, this derives [`PartialEq`] but
/// not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSnapshot {
    style: ComputedStyle,
    version: VersionStamp,
}

impl StyleSnapshot {
    /// Creates a snapshot of `style` stamped with `version`.
    #[must_use]
    pub fn new(style: ComputedStyle, version: VersionStamp) -> Self {
        Self { style, version }
    }

    /// Returns the snapshotted computed style.
    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    /// Returns the version stamp this snapshot was produced at.
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    /// Returns the computed value for `property`, delegating to the wrapped
    /// [`ComputedStyle`].
    #[must_use]
    pub fn get(&self, property: PropertyId) -> Option<&ComputedValue> {
        self.style.get(property)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn exposes_style_and_version() {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(StyleColor::BLACK));
        let snapshot = StyleSnapshot::new(style.clone(), VersionStamp::from_raw(7));

        assert_eq!(snapshot.style(), &style);
        assert_eq!(snapshot.version(), VersionStamp::from_raw(7));
        assert_eq!(
            snapshot.get(PropertyId::Color),
            Some(&ComputedValue::color(StyleColor::BLACK))
        );
    }
}
