//! Stroke end-cap style.

/// The shape drawn at the open ends of a stroked, unclosed path (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineCap {
    /// The stroke ends flush with the endpoint, with no cap extending past it.
    Butt,
    /// A semicircular cap of radius half the stroke width extends past the
    /// endpoint.
    Round,
    /// A square cap of half the stroke width extends past the endpoint.
    Square,
}

impl Default for LineCap {
    /// The default line cap is [`LineCap::Butt`].
    fn default() -> Self {
        Self::Butt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_butt() {
        assert_eq!(LineCap::default(), LineCap::Butt);
    }
}
