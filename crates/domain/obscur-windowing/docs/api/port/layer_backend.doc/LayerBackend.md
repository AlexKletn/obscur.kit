The port a platform implements to place a surface on the desktop rather than
in the window stack.

The sibling of [`WindowBackend`](crate::port::WindowBackend), and shaped the
same way on purpose: settings in, typed outcomes back, capabilities answerable
before anything is attempted, and events polled rather than pushed.

**What it does not carry, and why.** No render target and no loop handle. A
surface a GPU can draw into is a raw platform handle, and the crates that speak
that language are barred from the pure layers; the same is already true of
`WindowBackend`, and the same seam — the adapter hands its own concrete target
to whichever renderer is compiled in — applies here. The port covers what is
genuinely portable: what the application asked for, what the platform allowed,
how big the result is, and what has happened to it since.

**Honesty is the contract.** A backend answers
[`supports`](Self::supports) for every [`LayerFeature`], and a request it can
only partly meet comes back
[`conditional`](crate::request::WindowOperationResult::conditional) rather than
[`native`](crate::request::WindowOperationResult::native). Reporting success
for work it did not do is the one failure this port exists to prevent.
