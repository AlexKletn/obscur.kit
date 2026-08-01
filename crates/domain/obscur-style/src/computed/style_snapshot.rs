#![doc = include_str!("style_snapshot.doc/_module.md")]

use crate::computed::{ComputedStyle, ComputedValue};
use crate::property::PropertyId;
use obscur_identity::VersionStamp;

#[doc = include_str!("style_snapshot.doc/StyleSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSnapshot {
    style: ComputedStyle,
    version: VersionStamp,
}

impl StyleSnapshot {
    #[doc = include_str!("style_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(style: ComputedStyle, version: VersionStamp) -> Self {
        Self { style, version }
    }

    #[doc = include_str!("style_snapshot.doc/style.md")]
    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    #[doc = include_str!("style_snapshot.doc/version.md")]
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    #[doc = include_str!("style_snapshot.doc/get.md")]
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
