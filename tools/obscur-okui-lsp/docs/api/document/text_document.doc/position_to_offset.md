Maps a zero-based [`Position`] to an absolute byte offset.

The line is clamped to the last line and the character is clamped to the
end of its line (including any trailing newline byte), so the result is
always a valid offset in `0..=text_len`.
