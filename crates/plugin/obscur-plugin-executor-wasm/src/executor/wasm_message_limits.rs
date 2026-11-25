/// Host-side byte budgets for one guest message exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmMessageLimits {
    /// Maximum request payload in bytes.
    pub request_bytes: usize,
    /// Maximum response payload in bytes.
    pub response_bytes: usize,
}
