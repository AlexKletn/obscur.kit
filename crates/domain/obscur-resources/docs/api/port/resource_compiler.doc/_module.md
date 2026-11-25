The hexagonal port for compiling a CPU representation into a backend/GPU
artifact (spec §5.15).

This module defines [`ResourceCompiler`], the port that turns a decoded CPU
representation into a backend/GPU artifact ready for the runtime and its
backend/GPU cache (spec §5.15 compile). Because the GPU artifact itself is
adapter-specific — a texture, an uploaded font atlas, a compiled shader
pipeline — the domain never holds it directly: the compiler returns the
colocated [`CompiledResource`] descriptor, which records the artifact's
backend footprint (for cache budgeting) and its [`ResourceMetadata`]. The
colocated [`NullCompiler`] is the honest default that compiles nothing.

Compilation is modeled **synchronously** at the domain level: an adapter that
uploads on a render thread wraps its own async and calls this port with the
result.
