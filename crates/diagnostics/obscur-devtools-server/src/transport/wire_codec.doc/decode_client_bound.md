Decodes a server→client envelope, reversing
[`encode_client_bound`](Self::encode_client_bound).

# Errors

Returns a [`WireError`] when the buffer is truncated, holds invalid
UTF-8, or carries an unknown discriminant tag.
