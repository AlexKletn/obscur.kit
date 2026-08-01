Splits `text` into whitespace-delimited tokens, pairing each with its byte
offset within `text`.

Tokens are runs of non-[`char::is_ascii_whitespace`] characters; the offset is
the byte position where the run starts. The scan is left-to-right, so the
order is deterministic.
