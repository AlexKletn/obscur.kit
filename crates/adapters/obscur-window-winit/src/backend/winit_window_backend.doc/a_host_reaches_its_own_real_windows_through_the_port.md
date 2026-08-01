The claim the whole fix rests on: a host can create windows through the
port and get back the platform handles it needs to present to — and each
handle is *the window the handler materialized for that id*, not some
window and not a fabrication.

Two windows, and the identity assertion is deliberately not `is_some()`.
`winit_to_domain` is the map that routes native events back to a domain id,
so proving each returned handle's winit id is the key that maps back to the
id it was asked for proves the host's swapchain and the backend's event
routing name the same OS window. A `native_window` that ignored its `id` and
handed out *a* live window would satisfy every `is_some()` here and still
fail — which is what the second window buys.
