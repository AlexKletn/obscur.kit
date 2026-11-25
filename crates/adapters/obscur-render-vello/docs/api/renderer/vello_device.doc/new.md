Builds a Vello renderer on a host-provided `device`/`queue`.

# Errors

Returns [`VelloRenderError::DeviceCreation`] if [`vello::Renderer::new`]
fails (for example, the device lacks a feature Vello requires).
