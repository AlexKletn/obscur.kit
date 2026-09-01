A subscription that lasts as long as this value does.

Dropping it stops the watch — the thread, the child process, whatever the source
needed. That is the whole design: an application that forgets
[`stop`](Self::stop) still leaks nothing, and one that wants to stop early
does not have to keep a second handle to do it with.

Opaque on purpose. What a source needs to tear down differs — a monitor process
here, nothing at all elsewhere — and a caller that could see the difference
would end up depending on it.
