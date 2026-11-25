//! Tests generated lifecycle callbacks as an SDK consumer.

use obscur_plugin_abi::value::{AbiBuffer, AbiCallResult, AbiSlice};
use obscur_plugin_sdk::{
    export_native_plugin,
    native::{AbiHandle, AbiStatus, HostFunctionTable},
};

fn initialize(handle: AbiHandle) -> AbiStatus {
    assert_eq!(handle.raw(), 17);
    AbiStatus::OK
}

fn deactivate(_: AbiHandle) -> AbiStatus {
    AbiStatus::REJECTED
}

fn bind(handle: AbiHandle, host: HostFunctionTable) -> AbiStatus {
    let mut reply = [0; 4];
    // SAFETY: the test host table only operates on the provided buffers.
    match unsafe { host.exchange(handle, b"ping", &mut reply) } {
        Ok(b"ping") => AbiStatus::OK,
        _ => AbiStatus::REJECTED,
    }
}

unsafe extern "C" fn echo(
    handle: AbiHandle,
    request: AbiSlice,
    output: AbiBuffer,
) -> AbiCallResult {
    let result = || {
        if handle.raw() != 17 {
            return None;
        }
        // SAFETY: the caller passes exclusive reply storage and a live request slice.
        let bytes = unsafe { request.as_bytes(4)? };
        // SAFETY: output is writable and disjoint from the request for this callback.
        unsafe { output.write(bytes) }.then_some(bytes.len())
    };
    match result() {
        Some(written) => AbiCallResult {
            status: AbiStatus::OK,
            written,
        },
        None => AbiCallResult {
            status: AbiStatus::REJECTED,
            written: 0,
        },
    }
}

export_native_plugin!(initialize, deactivate, host = bind);

fn dispatch(handle: AbiHandle, bytes: &[u8]) -> Result<Vec<u8>, AbiStatus> {
    assert_eq!(handle.raw(), 17);
    Ok(bytes.to_vec())
}

obscur_plugin_sdk::export_native_dispatch!(dispatch);

#[test]
fn dispatches_a_message_and_contains_callback_panics() {
    let input = AbiSlice::from_bytes(b"ping");
    let mut bytes = [0; 4];
    let output = AbiBuffer::from_bytes(&mut bytes);
    // SAFETY: the two buffers are live and disjoint.
    let result = unsafe { obscur_plugin_dispatch_v1(AbiHandle::new(17), input, output) };
    assert_eq!(result.status, AbiStatus::OK);
    assert_eq!(result.written, 4);
    assert_eq!(&bytes, b"ping");
    let output = AbiBuffer::from_bytes(&mut bytes);
    // SAFETY: the input and output still borrow live, disjoint storage.
    let result = unsafe { obscur_plugin_dispatch_v1(AbiHandle::new(18), input, output) };
    assert_eq!(result.status, AbiStatus::PANICKED);
    assert_eq!(result.written, 0);
}

#[test]
fn binds_the_host_table_and_exchanges_borrowed_bytes() {
    let mut table = HostFunctionTable::new(echo);
    assert_eq!(
        obscur_plugin_bind_host_v1(AbiHandle::new(17), table),
        AbiStatus::OK
    );
    assert_eq!(
        obscur_plugin_bind_host_v1(AbiHandle::new(18), table),
        AbiStatus::REJECTED
    );
    table.byte_size = 0;
    assert_eq!(
        obscur_plugin_bind_host_v1(AbiHandle::new(17), table),
        AbiStatus::REJECTED
    );
}

#[test]
fn exports_valid_c_callbacks_and_converts_panics() {
    let lifecycle = obscur_plugin_lifecycle_v1();
    assert!(lifecycle.supports_v1());
    let start = lifecycle.initialize.unwrap();
    assert_eq!(start(AbiHandle::new(17)), AbiStatus::OK);
    assert_eq!(start(AbiHandle::new(18)), AbiStatus::PANICKED);
    assert_eq!(
        lifecycle.deactivate.unwrap()(AbiHandle::new(17)),
        AbiStatus::REJECTED
    );
}
