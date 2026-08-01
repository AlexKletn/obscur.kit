#![doc = include_str!("clock.doc/_module.md")]

use super::Instant;

#[doc = include_str!("clock.doc/Clock.md")]
pub trait Clock {
    #[doc = include_str!("clock.doc/now.md")]
    fn now(&self) -> Instant;
}
