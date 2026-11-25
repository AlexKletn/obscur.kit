A typed failure raised inside the Vello/wgpu backend (spec §5.2, §2.7).

Spec §5.2 requires backend failures to surface as explicit diagnostics rather
than silent fallbacks or panics. This enum names the distinct causes the
Vello adapter can hit while bringing up wgpu, configuring a surface, or
submitting a frame. It is local to this crate (§2.7): callers outside the
adapter see only the domain [`RendererError`] produced by
[`VelloRenderError::into_renderer_error`].

The mapping into [`RendererError`] treats an *unusable device* — creation
failure or a total absence of a GPU adapter — as a
[`RendererError::DeviceLost`], since in both cases there is no device to draw
with. Surface and frame failures map to the generic
[`RendererError::Backend`], each carrying its human-readable reason.
