/// Host-specified budgets for one isolated module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmLimits {
    /// Maximum verified entrypoint size in bytes.
    pub module_bytes: usize,
    /// Maximum linear memory in bytes.
    pub memory_bytes: usize,
    /// Maximum number of elements in the guest table.
    pub table_elements: usize,
    /// Fuel available to each invocation.
    pub fuel_per_call: u64,
}
