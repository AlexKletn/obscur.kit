//! Identity of a cached artifact (spec §5.19 caches, §5.20 content-addressing).

use obscur_identity::ResourceId;

#[doc = include_str!("../../docs/api/cache/cache_key.doc/CacheKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CacheKey {
    /// The resource this artifact was derived from.
    resource: ResourceId,
    /// A discriminator over the derivation parameters or content (spec §5.20).
    variant: u64,
}

impl CacheKey {
    /// Creates a key for a specific derivative of `resource`, discriminated by
    /// `variant` (spec §5.19).
    #[must_use]
    pub const fn new(resource: ResourceId, variant: u64) -> Self {
        Self { resource, variant }
    }

    /// Creates a key for the canonical, unparameterised artifact of `resource`
    /// (variant `0`).
    #[must_use]
    pub const fn for_resource(resource: ResourceId) -> Self {
        Self::new(resource, 0)
    }

    /// Creates a content-addressed key, using a hash of the artifact's content as
    /// the discriminator (spec §5.20 *content-addressed where useful*).
    #[must_use]
    pub const fn content_addressed(resource: ResourceId, content_hash: u64) -> Self {
        Self::new(resource, content_hash)
    }

    /// Returns the resource this artifact was derived from.
    #[must_use]
    pub const fn resource(self) -> ResourceId {
        self.resource
    }

    /// Returns the variant discriminator (spec §5.20).
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
