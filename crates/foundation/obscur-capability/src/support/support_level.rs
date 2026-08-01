#![doc = include_str!("support_level.doc/_module.md")]

#[doc = include_str!("support_level.doc/SupportLevel.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    #[doc = include_str!("support_level.doc/Native.md")]
    Native,
    #[doc = include_str!("support_level.doc/Emulated.md")]
    Emulated,
    #[doc = include_str!("support_level.doc/Limited.md")]
    Limited,
    #[doc = include_str!("support_level.doc/Unavailable.md")]
    Unavailable,
}
