#![doc = include_str!("frame_id.doc/_module.md")]

#[doc = include_str!("frame_id.doc/FrameId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FrameId(u64);

impl FrameId {
    #[doc = include_str!("frame_id.doc/FIRST.md")]
    pub const FIRST: Self = Self(0);

    #[doc = include_str!("frame_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("frame_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[doc = include_str!("frame_id.doc/next.md")]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_is_zero_and_default() {
        assert_eq!(FrameId::FIRST.raw(), 0);
        assert_eq!(FrameId::default(), FrameId::FIRST);
    }

    #[test]
    fn from_raw_round_trips() {
        assert_eq!(FrameId::from_raw(42).raw(), 42);
    }

    #[test]
    fn next_advances_monotonically() {
        let a = FrameId::FIRST;
        let b = a.next();
        let c = b.next();
        assert_eq!(b.raw(), 1);
        assert_eq!(c.raw(), 2);
        assert!(a < b && b < c);
    }

    #[test]
    fn next_wraps_at_max_without_panicking() {
        let max = FrameId::from_raw(u64::MAX);
        assert_eq!(max.next(), FrameId::FIRST);
    }
}
