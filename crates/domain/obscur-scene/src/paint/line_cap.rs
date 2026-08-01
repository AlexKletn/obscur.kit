#![doc = include_str!("line_cap.doc/_module.md")]

#[doc = include_str!("line_cap.doc/LineCap.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineCap {
    #[doc = include_str!("line_cap.doc/Butt.md")]
    Butt,
    #[doc = include_str!("line_cap.doc/Round.md")]
    Round,
    #[doc = include_str!("line_cap.doc/Square.md")]
    Square,
}

impl Default for LineCap {
    #[doc = include_str!("line_cap.doc/default.md")]
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
