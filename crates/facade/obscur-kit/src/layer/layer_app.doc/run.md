Opens the surface and runs until the compositor closes it.

# Errors

[`LayerAppError::Unsupported`] where the compositor has no layer shell,
[`LayerAppError::Shell`] for a connection failure,
[`LayerAppError::Gpu`] where no adapter can be acquired, and
[`LayerAppError::NoRoot`] when no root view was set.
