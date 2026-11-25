#![doc = include_str!("../../docs/api/port/resource_decoder.doc/_module.md")]

use crate::handle::ResourceKind;
use crate::state::ResourceError;

use super::DecodedResource;

#[doc = include_str!("../../docs/api/port/resource_decoder.doc/ResourceDecoder.md")]
pub trait ResourceDecoder {
    /// Returns whether this decoder can decode resources of `kind` (spec §5.15).
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    #[doc = include_str!("../../docs/api/port/resource_decoder.doc/decode.md")]
    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::NullDecoder;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceDecoder>`.
    #[test]
    fn resource_decoder_is_object_safe() {
        let decoder: Box<dyn ResourceDecoder> = Box::new(NullDecoder::new());
        assert!(!decoder.supports(ResourceKind::Image));
    }
}
