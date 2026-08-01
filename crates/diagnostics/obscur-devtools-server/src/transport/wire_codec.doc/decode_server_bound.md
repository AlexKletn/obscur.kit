Decodes a client→server envelope, reversing
[`encode_server_bound`](Self::encode_server_bound).

# Errors

Returns a [`WireError`] when the buffer is truncated, holds invalid
UTF-8, or carries an unknown discriminant tag.
