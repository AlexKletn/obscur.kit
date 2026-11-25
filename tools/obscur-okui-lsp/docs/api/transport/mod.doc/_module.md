The LSP base-protocol transport: `Content-Length`-framed messages over any
byte stream.

The framing is the header block defined by the LSP specification — one or
more `Header: value\r\n` lines (only `Content-Length` is significant),
terminated by a blank `\r\n`, followed by exactly that many bytes of UTF-8
JSON. Both helpers work over any [`BufRead`](std::io::BufRead) /
[`Write`](std::io::Write), so the server can be driven by real stdio or by
in-memory buffers in tests.
