A [`ResourceDecoder`] that decodes any image into a 2×2 opaque RGBA buffer with a
2×2 intrinsic size.

The counterpart to [`CannedImageLoader`](crate::mock::CannedImageLoader): it
ignores the bytes it is given, so a test can assert that an image resolved and
painted without shipping an encoded asset to decode.
