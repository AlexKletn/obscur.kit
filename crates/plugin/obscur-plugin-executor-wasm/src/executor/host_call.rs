use wasmi::{Caller, Extern, StoreLimits};

pub(super) const MODULE: &str = "obscur_host";
pub(super) const SUBMIT: &str = "submit_v1";

pub(super) struct WasmState {
    pub(super) limits: StoreLimits,
    pub(super) max_call_bytes: Option<usize>,
    pub(super) pending_call: Option<Vec<u8>>,
}

pub(super) fn submit(mut caller: Caller<'_, WasmState>, ptr: i32, len: i32) -> i32 {
    const ACCEPTED: i32 = 0;
    const INVALID: i32 = 1;
    const BUSY: i32 = 2;

    let state = caller.data();
    if state.pending_call.is_some() {
        return BUSY;
    }
    let (Ok(start), Ok(len)) = (usize::try_from(ptr), usize::try_from(len)) else {
        return INVALID;
    };
    if len == 0 || state.max_call_bytes.is_none_or(|max| len > max) {
        return INVALID;
    }
    let Some(end) = start.checked_add(len) else {
        return INVALID;
    };
    let Some(memory) = caller.get_export("memory").and_then(Extern::into_memory) else {
        return INVALID;
    };
    let Some(bytes) = memory.data(&caller).get(start..end) else {
        return INVALID;
    };
    let packet = bytes.to_vec();
    caller.data_mut().pending_call = Some(packet);
    ACCEPTED
}
