Descriptive metadata about a loaded resource (spec §5.16).

Captures the intrinsic, backend-agnostic properties a ready resource exposes:
its byte size, MIME type, intrinsic pixel size and DPI-variant scale. Per spec
§5.16, images carry an intrinsic size and thus an aspect ratio; non-visual
resources such as CSS leave [`ResourceMetadata::intrinsic_size`] as `None`.

Because it holds `f32`/[`Size`] fields it derives only `Debug`, `Clone`, and
`PartialEq` (floats preclude `Eq`).
