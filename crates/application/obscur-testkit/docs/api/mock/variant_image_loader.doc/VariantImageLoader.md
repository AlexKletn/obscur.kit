A [`ResourceLoader`] over a fake filesystem of `path -> pixel dimension` that
records every path it is asked for (spec §5.16 DPI variants).

The "encoded bytes" of an asset are a single byte holding its square pixel
dimension, which [`VariantImageDecoder`](crate::mock::VariantImageDecoder) turns
back into an image of that size — so a test can tell *which file* was loaded by
looking at the decoded image's pixel width. A path not in the map is
[`ResourceError::NotFound`], exactly as a project that ships no `@2x` asset
behaves.

[`asked`](Self::asked) is the other half of the answer, and often the sharper
one: at base density the `@2x` file must not even be *asked* for, which is how a
1x application pays nothing for the feature.
