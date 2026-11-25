//! Native lifecycle table and panic conversion.

use obscur_plugin_abi::{
    boundary::PanicBoundary,
    function_table::PluginFunctionTable,
    value::{AbiHandle, AbiStatus},
};

extern "C" fn start(handle: AbiHandle) -> AbiStatus {
    PanicBoundary::status(|| match handle.raw() {
        1 => AbiStatus::OK,
        _ => panic!("guest callback"),
    })
}

#[test]
fn validates_lifecycle_callbacks_and_contains_panics() {
    let mut table = PluginFunctionTable::new(start, start);
    assert!(table.supports_v1());
    let callback = table.initialize.unwrap();
    assert_eq!(callback(AbiHandle::new(1)), AbiStatus::OK);
    assert_eq!(callback(AbiHandle::new(2)), AbiStatus::PANICKED);
    table.byte_size = 8;
    assert!(!table.supports_v1());
    table = PluginFunctionTable::new(start, start);
    table.abi_version = 2;
    assert!(!table.supports_v1());
    table = PluginFunctionTable::new(start, start);
    table.deactivate = None;
    assert!(!table.supports_v1());
}
