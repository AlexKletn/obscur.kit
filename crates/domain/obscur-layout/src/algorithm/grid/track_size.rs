#![doc = include_str!("track_size.doc/_module.md")]

#[doc = include_str!("track_size.doc/TrackSize.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSize {
    #[doc = include_str!("track_size.doc/Fixed.md")]
    Fixed(f32),
    #[doc = include_str!("track_size.doc/Percent.md")]
    Percent(f32),
    #[doc = include_str!("track_size.doc/Fr.md")]
    Fr(f32),
    #[doc = include_str!("track_size.doc/Auto.md")]
    Auto,
    #[doc = include_str!("track_size.doc/MinContent.md")]
    MinContent,
    #[doc = include_str!("track_size.doc/MaxContent.md")]
    MaxContent,
    #[doc = include_str!("track_size.doc/MinMax.md")]
    MinMax(Box<TrackSize>, Box<TrackSize>),
}

impl TrackSize {
    #[doc = include_str!("track_size.doc/minmax.md")]
    #[must_use]
    pub fn minmax(min: TrackSize, max: TrackSize) -> Self {
        TrackSize::MinMax(Box::new(min), Box::new(max))
    }
}

#[cfg(test)]
mod tests {
    use super::TrackSize;

    #[test]
    fn minmax_boxes_bounds() {
        let t = TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0));
        assert_eq!(
            t,
            TrackSize::MinMax(
                Box::new(TrackSize::Fixed(100.0)),
                Box::new(TrackSize::Fr(1.0))
            )
        );
    }

    #[test]
    fn variants_are_equatable_and_clonable() {
        let a = TrackSize::Auto;
        assert_eq!(a.clone(), TrackSize::Auto);
        assert_ne!(TrackSize::Fr(1.0), TrackSize::Fr(2.0));
        assert_ne!(TrackSize::Fixed(10.0), TrackSize::Auto);
        assert_eq!(TrackSize::Percent(0.30).clone(), TrackSize::Percent(0.30));
        assert_ne!(TrackSize::Percent(0.30), TrackSize::Percent(0.50));
        assert_ne!(TrackSize::Percent(0.50), TrackSize::Fixed(0.50));
    }
}
