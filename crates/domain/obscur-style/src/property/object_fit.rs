#![doc = include_str!("object_fit.doc/_module.md")]

#[doc = include_str!("object_fit.doc/ObjectFit.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectFit {
    #[doc = include_str!("object_fit.doc/Fill.md")]
    #[default]
    Fill,
    #[doc = include_str!("object_fit.doc/Contain.md")]
    Contain,
    #[doc = include_str!("object_fit.doc/Cover.md")]
    Cover,
    #[doc = include_str!("object_fit.doc/None.md")]
    None,
    #[doc = include_str!("object_fit.doc/ScaleDown.md")]
    ScaleDown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_fill() {
        assert_eq!(ObjectFit::default(), ObjectFit::Fill);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(ObjectFit::Fill, ObjectFit::Contain);
        assert_ne!(ObjectFit::Cover, ObjectFit::None);
        assert_ne!(ObjectFit::None, ObjectFit::ScaleDown);
    }
}
