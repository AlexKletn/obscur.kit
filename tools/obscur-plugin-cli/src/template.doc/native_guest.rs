use obscur_plugin_sdk::{
    export_native_dispatch, export_native_plugin,
    native::{AbiHandle, AbiStatus},
};

fn initialize(_: AbiHandle) -> AbiStatus {
    AbiStatus::OK
}

fn deactivate(_: AbiHandle) -> AbiStatus {
    AbiStatus::OK
}

export_native_plugin!(initialize, deactivate);

fn dispatch(_: AbiHandle, request: &[u8]) -> Result<Vec<u8>, AbiStatus> {
    Ok(request.to_vec())
}

export_native_dispatch!(dispatch);
