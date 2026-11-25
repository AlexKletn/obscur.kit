//! [`ResourceOutcome`]: what one finished resource-load task produced, and the
//! load that produces it.

use obscur_resources::handle::{ResourceHandle, ResourceKind};
use obscur_resources::port::{ResourceDecoder, ResourceLoader, VectorDecoder};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceVersion};
use obscur_scene::color::AlphaMode;
use obscur_scene::paint::{SceneImage, SceneVector};

#[doc = include_str!("../../docs/api/runtime/resource_outcome.doc/ResourceOutcome.md")]
#[derive(Debug)]
pub(crate) enum ResourceOutcome {
    /// A raster image load, keyed by the authored URL/path.
    Image {
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
        result: Result<SceneImage, ResourceError>,
    },
    /// A vector document load, keyed the same way.
    Vector {
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
        result: Result<SceneVector, ResourceError>,
    },
}

impl ResourceOutcome {
    /// Constructs a failed outcome with the identity of the original resource request.
    pub(crate) fn failed(
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
        error: ResourceError,
    ) -> Self {
        if handle.kind() == ResourceKind::Svg {
            Self::Vector {
                key,
                handle,
                version,
                result: Err(error),
            }
        } else {
            Self::Image {
                key,
                handle,
                version,
                result: Err(error),
            }
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_outcome.doc/load_image.md")]
    pub(crate) fn load_image(
        loader: &dyn ResourceLoader,
        decoder: &dyn ResourceDecoder,
        candidates: &[(Box<str>, f32)],
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
    ) -> Self {
        Self::Image {
            key,
            handle,
            version,
            result: load_and_decode_image(loader, decoder, candidates),
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_outcome.doc/load_vector.md")]
    pub(crate) fn load_vector(
        loader: &dyn ResourceLoader,
        decoder: &dyn VectorDecoder,
        source: &ResourceSource,
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
    ) -> Self {
        Self::Vector {
            key,
            handle,
            version,
            result: loader
                .load(source)
                .and_then(|loaded| decoder.decode_vector(loaded.bytes())),
        }
    }
}

#[doc = include_str!("../../docs/api/runtime/resource_outcome.doc/load_and_decode_image.md")]
fn load_and_decode_image(
    loader: &dyn ResourceLoader,
    decoder: &dyn ResourceDecoder,
    candidates: &[(Box<str>, f32)],
) -> Result<SceneImage, ResourceError> {
    let mut last_error = ResourceError::NotFound;
    for (path, density) in candidates {
        match decode_one_image(loader, decoder, &ResourceSource::file(path.clone())) {
            Ok(image) => return Ok(image.with_scale(*density)),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

/// Loads and decodes exactly one image source — one rung of the
/// [`load_and_decode_image`] ladder.
fn decode_one_image(
    loader: &dyn ResourceLoader,
    decoder: &dyn ResourceDecoder,
    source: &ResourceSource,
) -> Result<SceneImage, ResourceError> {
    let loaded = loader.load(source)?;
    let decoded = decoder.decode(ResourceKind::Image, loaded.bytes())?;
    // The decoder records the intrinsic pixel size in its metadata; the buffer is
    // width*height*4 RGBA8 bytes.
    let size = decoded.metadata().intrinsic_size().ok_or_else(|| {
        ResourceError::DecodeFailed("decoded image carries no intrinsic size".into())
    })?;
    if size.width == 0.0 || size.height == 0.0 {
        return Err(ResourceError::DecodeFailed(
            "decoded image has zero extent".into(),
        ));
    }
    // Float-to-integer casts saturate and truncate. Validate the decoder's
    // pixel counts first so the byte-length check uses the actual dimensions.
    let valid_extent = |extent: f32| {
        extent >= 1.0 && f64::from(extent) <= f64::from(u32::MAX) && extent.fract() == 0.0
    };
    if !valid_extent(size.width) || !valid_extent(size.height) {
        return Err(ResourceError::DecodeFailed(
            "decoded image intrinsic size is not a valid pixel extent".into(),
        ));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = size.width as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = size.height as u32;
    SceneImage::new(width, height, decoded.data().clone(), AlphaMode::Straight).ok_or_else(|| {
        ResourceError::DecodeFailed("decoded image bytes do not match its intrinsic size".into())
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use obscur_geometry::Size;
    use obscur_identity::ResourceId;
    use obscur_resources::handle::{ResourceHandle, ResourceKind};
    use obscur_resources::port::{DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader};
    use obscur_resources::source::{ResourceSource, dpi_variants};
    use obscur_resources::state::{ResourceError, ResourceMetadata, ResourceVersion};

    use super::{ResourceOutcome, load_and_decode_image};

    /// A loader that resolves exactly the paths it was given and reports every
    /// other one as unavailable, naming the path it refused — which is what lets
    /// a test tell *which* rung of the ladder produced the error it sees.
    struct PathLoader(Vec<&'static str>);

    impl ResourceLoader for PathLoader {
        fn can_load(&self, source: &ResourceSource) -> bool {
            matches!(source, ResourceSource::File(path) if self.0.contains(&&**path))
        }

        fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
            let ResourceSource::File(path) = source else {
                return Err(ResourceError::NotFound);
            };
            if !self.0.contains(&&**path) {
                return Err(ResourceError::SourceUnavailable(path.clone()));
            }
            Ok(LoadedBytes::new(
                Arc::from(&b"encoded"[..]),
                ResourceMetadata::new(),
            ))
        }
    }

    /// A decoder that answers every call with one 1x1 RGBA8 pixel, and whose
    /// reported intrinsic size can be made to disagree with that buffer.
    struct OnePixelDecoder {
        claimed: Size,
    }

    impl ResourceDecoder for OnePixelDecoder {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            let bytes: &[u8] = if self.claimed.width == 0.0 || self.claimed.height == 0.0 {
                &[]
            } else {
                &[0xFF, 0x00, 0x00, 0xFF]
            };
            Ok(DecodedResource::new(
                Arc::from(bytes),
                ResourceMetadata::new().with_intrinsic_size(self.claimed),
            ))
        }
    }

    fn one_pixel() -> OnePixelDecoder {
        OnePixelDecoder {
            claimed: Size::new(1.0, 1.0),
        }
    }

    #[test]
    fn the_densest_variant_that_resolves_wins_and_carries_its_density() {
        let candidates = dpi_variants("logo.png", 2.0);
        let both = PathLoader(vec!["logo@2x.png", "logo.png"]);
        let image = load_and_decode_image(&both, &one_pixel(), &candidates)
            .expect("a resolvable ladder decodes");
        assert!((image.scale() - 2.0).abs() < f32::EPSILON);
        // The `@2x` pixel is one pixel of a half-sized logical box, which is the
        // whole point of tagging the density.
        assert_eq!(image.logical_size(), (0.5, 0.5));
    }

    #[test]
    fn a_missing_variant_falls_through_to_the_authored_path() {
        let candidates = dpi_variants("logo.png", 3.0);
        let base_only = PathLoader(vec!["logo.png"]);
        let image = load_and_decode_image(&base_only, &one_pixel(), &candidates)
            .expect("the authored path still resolves");
        assert!((image.scale() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn an_unresolvable_ladder_reports_the_authored_paths_error() {
        let candidates = dpi_variants("logo.png", 3.0);
        let nothing = PathLoader(Vec::new());
        let error = load_and_decode_image(&nothing, &one_pixel(), &candidates)
            .expect_err("no candidate resolves");
        // Not `logo@3x.png` or `logo@2x.png`: naming a variant the author never
        // wrote would send them looking for a file that was never meant to exist.
        assert_eq!(
            error,
            ResourceError::SourceUnavailable("logo.png".into()),
            "the last rung's error is the one worth reporting"
        );
    }

    #[test]
    fn a_buffer_that_does_not_match_the_claimed_size_is_a_decode_failure() {
        let candidates = dpi_variants("logo.png", 1.0);
        let loader = PathLoader(vec!["logo.png"]);
        // Four bytes claimed to be a 2x2 image: sixteen bytes short.
        let liar = OnePixelDecoder {
            claimed: Size::new(2.0, 2.0),
        };
        let error = load_and_decode_image(&loader, &liar, &candidates)
            .expect_err("the byte-length guard rejects it");
        assert!(matches!(error, ResourceError::DecodeFailed(_)));
    }

    /// Rejects zero and invalid host-reported pixel extents before constructing a scene image.
    #[test]
    fn invalid_image_extents_fail_before_caching() {
        let candidates = dpi_variants("logo.png", 1.0);
        let loader = PathLoader(vec!["logo.png"]);
        for claimed in [Size::new(0.0, 1.0), Size::new(1.0, 0.0)] {
            let error = load_and_decode_image(&loader, &OnePixelDecoder { claimed }, &candidates)
                .expect_err("zero pixels must fail even with a matching empty buffer");
            assert_eq!(
                error,
                ResourceError::DecodeFailed("decoded image has zero extent".into())
            );
        }

        for width in [f32::NAN, f32::INFINITY, -1.0, 1.5, 4_294_967_296.0] {
            let claimed = Size::new(width, 1.0);
            let error = load_and_decode_image(&loader, &OnePixelDecoder { claimed }, &candidates)
                .expect_err("invalid width must fail before integer conversion");
            assert_eq!(
                error,
                ResourceError::DecodeFailed(
                    "decoded image intrinsic size is not a valid pixel extent".into(),
                )
            );
        }
    }

    #[test]
    fn the_outcome_carries_the_identity_the_job_was_started_under() {
        let handle = ResourceHandle::new(ResourceId::from_raw(7), ResourceKind::Image);
        let version = ResourceVersion::initial();
        let outcome = ResourceOutcome::load_image(
            &PathLoader(vec!["logo.png"]),
            &one_pixel(),
            &dpi_variants("logo.png", 1.0),
            "logo.png".into(),
            handle,
            version,
        );
        // The hub rules on staleness, so the answer is worthless without the
        // handle and version it is an answer *for*.
        let ResourceOutcome::Image {
            key,
            handle: carried,
            version: carried_version,
            result,
        } = outcome
        else {
            panic!("a raster load is an image outcome");
        };
        assert_eq!(&*key, "logo.png");
        assert_eq!(carried, handle);
        assert_eq!(carried_version, version);
        assert!(result.is_ok());
    }
}
