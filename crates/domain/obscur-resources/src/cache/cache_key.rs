#![doc = include_str!("cache_key.doc/_module.md")]

use obscur_identity::ResourceId;

#[doc = include_str!("cache_key.doc/CacheKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CacheKey {
    #[doc = include_str!("cache_key.doc/resource.md")]
    resource: ResourceId,
    #[doc = include_str!("cache_key.doc/variant.md")]
    variant: u64,
}

impl CacheKey {
    #[doc = include_str!("cache_key.doc/new.md")]
    #[must_use]
    pub const fn new(resource: ResourceId, variant: u64) -> Self {
        Self { resource, variant }
    }

    #[doc = include_str!("cache_key.doc/for_resource.md")]
    #[must_use]
    pub const fn for_resource(resource: ResourceId) -> Self {
        Self::new(resource, 0)
    }

    #[doc = include_str!("cache_key.doc/content_addressed.md")]
    #[must_use]
    pub const fn content_addressed(resource: ResourceId, content_hash: u64) -> Self {
        Self::new(resource, content_hash)
    }

    #[doc = include_str!("cache_key.doc/resource_2.md")]
    #[must_use]
    pub const fn resource(self) -> ResourceId {
        self.resource
    }

    #[doc = include_str!("cache_key.doc/variant_2.md")]
    #[must_use]
    pub const fn variant(self) -> u64 {
        self.variant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(raw: u64) -> ResourceId {
        ResourceId::from_raw(raw)
    }

    #[test]
    fn accessors_expose_fields() {
        let key = CacheKey::new(res(7), 42);
        assert_eq!(key.resource(), res(7));
        assert_eq!(key.variant(), 42);
    }

    #[test]
    fn for_resource_is_variant_zero() {
        assert_eq!(CacheKey::for_resource(res(7)), CacheKey::new(res(7), 0));
        assert_eq!(CacheKey::for_resource(res(7)).variant(), 0);
    }

    #[test]
    fn content_addressed_uses_hash_as_variant() {
        let key = CacheKey::content_addressed(res(3), 0xABCD);
        assert_eq!(key.variant(), 0xABCD);
        assert_eq!(key.resource(), res(3));
    }

    #[test]
    fn ordering_is_by_resource_then_variant() {
        let a = CacheKey::new(res(1), 5);
        let b = CacheKey::new(res(1), 9);
        let c = CacheKey::new(res(2), 0);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn equal_keys_compare_equal() {
        assert_eq!(CacheKey::new(res(4), 1), CacheKey::new(res(4), 1));
        assert_ne!(CacheKey::new(res(4), 1), CacheKey::new(res(4), 2));
    }
}
