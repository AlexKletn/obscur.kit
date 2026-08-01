An honest reference [`ResourceCompiler`] that compiles nothing (spec §5.15).

Adapters provide the real compilers that upload textures or build pipelines;
`NullCompiler` is the honest default a hub uses before any compiler is
registered. It reports [`ResourceCompiler::supports`] `== false` for every
[`ResourceKind`] and returns [`ResourceError::CompileFailed`] from
[`ResourceCompiler::compile`]. Its refusals are correct behavior, not stubs
waiting to be filled in.
