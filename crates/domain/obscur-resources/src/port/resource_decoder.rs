//! The hexagonal port for decoding a resource into a CPU representation (spec
//! §5.15).
//!
//! This module defines [`ResourceDecoder`], the port that turns the raw encoded
//! bytes of a resource into a CPU-side representation (spec §5.15 decode) — for
//! an image, decoded RGBA pixels; for a font, a parsed face; for CSS, a parsed
//! stylesheet. Real decoders live in adapter crates (image, svg, font); at the
//! domain boundary the decoded representation is an opaque byte buffer plus its
//! [`ResourceMetadata`], carried by the colocated [`DecodedResource`] output. The
//! colocated [`NullDecoder`] is the honest default that supports nothing.
//!
//! Decoding is modeled **synchronously** at the domain level: an adapter that
//! decodes on a worker thread wraps its own async and calls this port with the
//! result.

use std::sync::Arc;

use crate::handle::ResourceKind;
use crate::state::{ResourceError, ResourceMetadata};

/// A decoded CPU representation of a resource plus its metadata (spec §5.15).
///
/// This is the output of [`ResourceDecoder::decode`]: the decoded CPU bytes (for
/// example the RGBA pixels of an image) held in an [`Arc<[u8]>`] so clones are
/// cheap and the buffer can be shared with the compile stage, paired with the
/// [`ResourceMetadata`] the decoder determined — including the intrinsic pixel
/// size for a visual resource. The domain treats the buffer as opaque bytes; the
/// adapter that produced it knows its layout.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedResource {
    data: Arc<[u8]>,
    metadata: ResourceMetadata,
}

impl DecodedResource {
    /// Creates a decoded resource from CPU `data` and its `metadata`.
    #[must_use]
    pub fn new(data: Arc<[u8]>, metadata: ResourceMetadata) -> Self {
        Self { data, metadata }
    }

    /// Returns the decoded CPU bytes (spec §5.15).
    #[must_use]
    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }

    /// Returns the metadata determined while decoding (spec §5.15), including the
    /// intrinsic size for a visual resource.
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

/// The hexagonal port for decoding encoded bytes into a CPU representation (spec
/// §5.15 decode).
///
/// An adapter implements this once per format family (image, svg, font, CSS),
/// and the domain stores it as `Box<dyn ResourceDecoder>`; the trait is
/// therefore deliberately **object-safe** — no generic methods and no
/// `Self`-returning methods — so decoders can be held polymorphically and
/// composed. A decoder reports failure through [`ResourceError`] rather than
/// panicking.
pub trait ResourceDecoder {
    /// Returns whether this decoder can decode resources of `kind` (spec §5.15).
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    /// Decodes `bytes` of the given `kind` into a CPU representation (spec
    /// §5.15).
    ///
    /// Returns [`DecodedResource`] on success, or a [`ResourceError`] describing
    /// the failure — [`ResourceError::Unsupported`] when this decoder handles no
    /// such kind, or [`ResourceError::DecodeFailed`] when the bytes are
    /// malformed.
    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError>;
}

/// An honest reference [`ResourceDecoder`] that decodes nothing (spec §5.15).
///
/// Real decoders live in adapter crates (image, svg, font); `NullDecoder` is the
/// honest default a hub uses before any decoder is registered. It reports
/// [`ResourceDecoder::supports`] `== false` for every [`ResourceKind`] and
/// returns [`ResourceError::Unsupported`] from [`ResourceDecoder::decode`]. Its
/// refusals are correct behavior, not stubs waiting to be filled in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullDecoder;

impl NullDecoder {
    /// Creates a [`NullDecoder`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceDecoder for NullDecoder {
    fn supports(&self, _kind: ResourceKind) -> bool {
        // Honest: a null decoder handles no kind.
        false
    }

    fn decode(&self, _kind: ResourceKind, _bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        Err(ResourceError::Unsupported("no decoder registered".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceDecoder>`.
    #[test]
    fn resource_decoder_is_object_safe() {
        let decoder: Box<dyn ResourceDecoder> = Box::new(NullDecoder::new());
        assert!(!decoder.supports(ResourceKind::Image));
    }

    #[test]
    fn null_decoder_supports_nothing() {
        let decoder = NullDecoder::new();
        for kind in [
            ResourceKind::Image,
            ResourceKind::Font,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Svg,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!decoder.supports(kind));
        }
    }

    #[test]
    fn null_decoder_decode_is_unsupported() {
        let decoder = NullDecoder::new();
        assert_eq!(
            decoder.decode(ResourceKind::Image, &[1u8, 2, 3]),
            Err(ResourceError::Unsupported("no decoder registered".into()))
        );
    }

    #[test]
    fn decoded_resource_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(4);
        let decoded = DecodedResource::new(Arc::from(&[0u8, 1, 2, 3][..]), meta.clone());
        assert_eq!(&**decoded.data(), &[0u8, 1, 2, 3]);
        assert_eq!(decoded.metadata(), &meta);
    }
}
