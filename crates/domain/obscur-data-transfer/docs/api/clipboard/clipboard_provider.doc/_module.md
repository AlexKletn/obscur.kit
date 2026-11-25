The replaceable clipboard port and its in-process reference implementation
(spec §6.20 data transfer — clipboard).

[`ClipboardProvider`] is the hexagonal port through which the pure
data-transfer domain drives a real system clipboard: an interface adapter
implements it once per platform and the domain holds it as
`Box<dyn ClipboardProvider>`. The [`MemoryClipboard`](super::MemoryClipboard) is a
deterministic, in-process reference implementation — the honest default for
headless runs and tests. System clipboard *history* is explicitly out of
scope (spec §6.20): a provider models only the current contents.
