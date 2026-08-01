A [`ResourceLoader`] that returns four fixed bytes for any file source, or fails
every source.

The bytes are meaningless — [`CannedImageDecoder`](crate::mock::CannedImageDecoder)
ignores them — because the question these tests ask is about the *path*, not the
payload: does a `url()` reach a loader at all, does a failure stay a failure, does
the absence of a provider paint nothing. The failing form exists so that the
error path is exercised by the same fixture as the success path, one constructor
apart.
