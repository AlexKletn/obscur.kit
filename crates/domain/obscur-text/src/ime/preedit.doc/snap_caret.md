Clamps `caret` to `text.len()` and snaps it down to the nearest UTF-8
character boundary, so the returned offset always indexes `text` safely.
