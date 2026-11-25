How far the in-progress [`read_frame`](Self::read_frame) has got.

A framed reader and a read timeout do not mix unless the reader is
**resumable**: a timeout can land after some of a frame has arrived, and
discarding those bytes would make the next call read the frame's own tail as
a length prefix — desyncing the stream permanently. Progress therefore lives
here, across calls, so a timed-out read resumes exactly where it stopped.
