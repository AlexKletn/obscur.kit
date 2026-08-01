#![doc = include_str!("stable_key.doc/_module.md")]

#[doc = include_str!("stable_key.doc/StableKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StableKey {
    #[doc = include_str!("stable_key.doc/Index.md")]
    Index(u64),
    #[doc = include_str!("stable_key.doc/Name.md")]
    Name(Box<str>),
    #[doc = include_str!("stable_key.doc/Symbol.md")]
    Symbol(super::InternedSymbol),
}

impl From<u64> for StableKey {
    fn from(value: u64) -> Self {
        Self::Index(value)
    }
}

impl From<&str> for StableKey {
    fn from(value: &str) -> Self {
        Self::Name(value.into())
    }
}
