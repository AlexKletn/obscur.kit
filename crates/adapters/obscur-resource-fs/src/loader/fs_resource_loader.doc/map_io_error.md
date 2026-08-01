Maps a filesystem [`std::io::Error`] onto a [`ResourceError`] (spec §5.15
failed state): a missing file is [`ResourceError::NotFound`], a permission
failure is a [`ResourceError::SourceUnavailable`] with a fixed reason, and any
other I/O failure is [`ResourceError::SourceUnavailable`] carrying the I/O
error's own message.
