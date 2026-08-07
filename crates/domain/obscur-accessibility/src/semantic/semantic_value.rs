#![doc = include_str!("semantic_value.doc/_module.md")]

#[doc = include_str!("semantic_value.doc/SemanticValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticValue {
    #[doc = include_str!("semantic_value.doc/None.md")]
    None,
    #[doc = include_str!("semantic_value.doc/Text.md")]
    Text(Box<str>),
    #[doc = include_str!("semantic_value.doc/Bool.md")]
    Bool(bool),
    #[doc = include_str!("semantic_value.doc/Number.md")]
    Number {
        #[doc = include_str!("semantic_value.doc/value.md")]
        value: f32,
        #[doc = include_str!("semantic_value.doc/min.md")]
        min: f32,
        #[doc = include_str!("semantic_value.doc/max.md")]
        max: f32,
    },
    #[doc = include_str!("semantic_value.doc/Range.md")]
    Range {
        #[doc = include_str!("semantic_value.doc/low.md")]
        low: f32,
        #[doc = include_str!("semantic_value.doc/high.md")]
        high: f32,
        #[doc = include_str!("semantic_value.doc/min.md")]
        min: f32,
        #[doc = include_str!("semantic_value.doc/max.md")]
        max: f32,
    },
}

impl SemanticValue {
    #[doc = include_str!("semantic_value.doc/none.md")]
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    #[doc = include_str!("semantic_value.doc/text.md")]
    #[must_use]
    pub fn text(text: impl Into<Box<str>>) -> Self {
        Self::Text(text.into())
    }

    #[doc = include_str!("semantic_value.doc/bool.md")]
    #[must_use]
    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    #[doc = include_str!("semantic_value.doc/number.md")]
    #[must_use]
    pub const fn number(value: f32, min: f32, max: f32) -> Self {
        Self::Number { value, min, max }
    }

    #[doc = include_str!("semantic_value.doc/range.md")]
    #[must_use]
    pub const fn range(low: f32, high: f32, min: f32, max: f32) -> Self {
        Self::Range {
            low,
            high,
            min,
            max,
        }
    }

    #[doc = include_str!("semantic_value.doc/is_none.md")]
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[doc = include_str!("semantic_value.doc/as_text.md")]
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    #[doc = include_str!("semantic_value.doc/as_number.md")]
    #[must_use]
    pub const fn as_number(&self) -> Option<f32> {
        match self {
            Self::Number { value, .. } => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("semantic_value.doc/as_range.md")]
    #[must_use]
    pub const fn as_range(&self) -> Option<(f32, f32)> {
        match self {
            Self::Range { low, high, .. } => Some((*low, *high)),
            _ => None,
        }
    }

    #[doc = include_str!("semantic_value.doc/ratio.md")]
    #[must_use]
    pub fn ratio(&self) -> f32 {
        match self {
            Self::Number { value, min, max } => {
                if max <= min {
                    0.0
                } else {
                    ((value - min) / (max - min)).clamp(0.0, 1.0)
                }
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn none_value() {
        let v = SemanticValue::none();
        assert!(v.is_none());
        assert_eq!(v.as_text(), None);
        assert_eq!(v.as_number(), None);
    }

    #[test]
    fn text_value() {
        let v = SemanticValue::text("hello");
        assert!(!v.is_none());
        assert_eq!(v.as_text(), Some("hello"));
        assert_eq!(v.as_number(), None);
    }

    #[test]
    fn bool_value() {
        let v = SemanticValue::bool(true);
        assert_eq!(v, SemanticValue::Bool(true));
        assert_eq!(v.as_text(), None);
    }

    #[test]
    fn number_as_number() {
        let v = SemanticValue::number(3.0, 0.0, 10.0);
        assert_eq!(v.as_number(), Some(3.0));
    }

    #[test]
    fn range_carries_both_ends_and_refuses_to_pass_for_one() {
        // The whole point of the variant: a two-thumb control has two positions
        // and no single one, so `as_number` must not hand back an end. A caller
        // that got `Some(20.0)` for a 20..80 range would announce half the
        // control as if it were the whole of it.
        let v = SemanticValue::range(20.0, 80.0, 0.0, 100.0);
        assert!(!v.is_none());
        assert_eq!(v.as_range(), Some((20.0, 80.0)));
        assert_eq!(v.as_number(), None);
        assert_eq!(SemanticValue::number(5.0, 0.0, 10.0).as_range(), None);
    }

    #[test]
    fn number_ratio_normalizes() {
        assert_eq!(SemanticValue::number(5.0, 0.0, 10.0).ratio(), 0.5);
        assert_eq!(SemanticValue::number(0.0, 0.0, 10.0).ratio(), 0.0);
        assert_eq!(SemanticValue::number(10.0, 0.0, 10.0).ratio(), 1.0);
    }

    #[test]
    fn number_ratio_clamps_out_of_range() {
        assert_eq!(SemanticValue::number(-5.0, 0.0, 10.0).ratio(), 0.0);
        assert_eq!(SemanticValue::number(20.0, 0.0, 10.0).ratio(), 1.0);
    }

    #[test]
    fn number_ratio_guards_degenerate_range() {
        assert_eq!(SemanticValue::number(5.0, 5.0, 5.0).ratio(), 0.0);
    }
}
