//! `PluginKind` (skeleton stub for the spec-defined entity; not yet implemented).

/// The execution class declared by a plugin package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    /// A trusted native library.
    Native,
    /// A WebAssembly module.
    Wasm,
    /// A separate local process.
    Process,
}
