use crate::handle::ResourceKind;
use crate::state::ResourceError;

use super::{DecodedResource, ResourceDecoder};

#[doc = include_str!("../../docs/api/port/null_decoder.doc/NullDecoder.md")]
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
}
