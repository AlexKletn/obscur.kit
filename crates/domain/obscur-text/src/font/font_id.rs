#![doc = include_str!("font_id.doc/_module.md")]

#[doc = include_str!("font_id.doc/FontId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FontId(u32);

impl FontId {
    #[doc = include_str!("font_id.doc/PRIMARY.md")]
    pub const PRIMARY: FontId = FontId(0);

    #[doc = include_str!("font_id.doc/new.md")]
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[doc = include_str!("font_id.doc/get.md")]
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    #[doc = include_str!("font_id.doc/index.md")]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_is_zero_and_default() {
        assert_eq!(FontId::PRIMARY.get(), 0);
        assert_eq!(FontId::PRIMARY.index(), 0);
        assert_eq!(FontId::default(), FontId::PRIMARY);
    }

    #[test]
    fn new_round_trips() {
        let id = FontId::new(7);
        assert_eq!(id.get(), 7);
        assert_eq!(id.index(), 7);
        assert_eq!(id, FontId::new(7));
        assert_ne!(id, FontId::PRIMARY);
    }

    #[test]
    fn ordering_follows_index() {
        assert!(FontId::PRIMARY < FontId::new(1));
        assert!(FontId::new(1) < FontId::new(2));
        let mut ids = [FontId::new(3), FontId::new(1), FontId::new(2)];
        ids.sort();
        assert_eq!(ids, [FontId::new(1), FontId::new(2), FontId::new(3)]);
    }
}
