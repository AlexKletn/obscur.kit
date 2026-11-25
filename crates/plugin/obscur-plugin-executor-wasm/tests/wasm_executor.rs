//! Verified package execution and guest resource boundaries.

use obscur_plugin_executor_wasm::{
    error::WasmExecutorError,
    executor::{WasmExecutor, WasmLimits},
};
use obscur_plugin_model::manifest::Version;
use obscur_plugin_package::{
    package::{PackageEntry, PackageReader, PluginPackage},
    path::PackagePath,
    validation::{PackageError, PackageLimits, PackageValidator},
};
use serde_json::json;
use sha2::{Digest, Sha256};

struct Entries(Vec<PackageEntry>);

impl PackageReader for Entries {
    fn read_entries(self, _: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
        Ok(self.0)
    }
}

fn package(kind: &str, bytes: Vec<u8>) -> PluginPackage {
    let path = "plugin.wasm";
    let manifest = json!({
        "schema_version": 1,
        "id": "example.wasm",
        "name": "Example",
        "author": "Example",
        "version": "1.0.0",
        "kind": kind,
        "compatibility": { "plugin_api": "^1", "ui_ir": "^1" },
        "entrypoint": { "path": path },
        "files": [{ "path": path, "sha256": format!("{:x}", Sha256::digest(&bytes)) }]
    });
    PackageValidator::new(Version::new(1, 0, 0), Version::new(1, 0, 0))
        .load(Entries(vec![
            PackageEntry::file(
                PackagePath::parse("manifest.json").unwrap(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
            PackageEntry::file(PackagePath::parse(path).unwrap(), bytes),
        ]))
        .unwrap()
}

fn limits() -> WasmLimits {
    WasmLimits {
        module_bytes: 4096,
        memory_bytes: 65536,
        table_elements: 4,
        fuel_per_call: 1000,
    }
}

#[test]
fn executes_typed_exports_from_a_verified_package() {
    let bytes = wat::parse_str(
        r#"(module (func (export "increment") (param i32) (result i32)
        local.get 0 i32.const 1 i32.add))"#,
    )
    .unwrap();
    let package = package("wasm", bytes);
    let mut executor = WasmExecutor::load(&package, limits()).unwrap();
    assert_eq!(
        executor.call_i32("missing", 0),
        Err(WasmExecutorError::InvalidExport)
    );
    assert_eq!(executor.call_i32("increment", 41), Ok(42));
    assert_eq!(executor.call_i32("increment", -2), Ok(-1));
}

#[test]
fn rejects_unavailable_imports_implicit_start_and_wrong_package_type() {
    assert!(matches!(
        WasmExecutor::load(&package("wasm", b"(module)".to_vec()), limits()),
        Err(WasmExecutorError::InvalidModule(_))
    ));
    let imports =
        wat::parse_str(r#"(module (import "wasi_snapshot_preview1" "fd_write" (func)))"#).unwrap();
    assert!(matches!(
        WasmExecutor::load(&package("wasm", imports), limits()),
        Err(WasmExecutorError::ImportsUnsupported)
    ));

    let start = wat::parse_str(r#"(module (func $start unreachable) (start $start))"#).unwrap();
    assert!(matches!(
        WasmExecutor::load(&package("wasm", start), limits()),
        Err(WasmExecutorError::InvalidModule(_))
    ));

    let empty = wat::parse_str("(module)").unwrap();
    assert!(matches!(
        WasmExecutor::load(&package("process", empty.clone()), limits()),
        Err(WasmExecutorError::WrongPluginKind)
    ));
    assert!(matches!(
        WasmExecutor::load(
            &package("wasm", empty),
            WasmLimits {
                module_bytes: 7,
                ..limits()
            }
        ),
        Err(WasmExecutorError::ModuleTooLarge)
    ));
}

#[test]
fn fuel_exhaustion_faults_only_the_current_instance() {
    let bytes = wat::parse_str(
        r#"(module
        (func (export "spin") (param i32) (result i32) (loop $again br $again) i32.const 0)
        (func (export "echo") (param i32) (result i32) local.get 0))"#,
    )
    .unwrap();
    let package = package("wasm", bytes);
    let mut executor = WasmExecutor::load(&package, limits()).unwrap();
    assert_eq!(
        executor.call_i32("spin", 0),
        Err(WasmExecutorError::OutOfFuel)
    );
    assert_eq!(
        executor.call_i32("echo", 1),
        Err(WasmExecutorError::Faulted)
    );
    let mut fresh = WasmExecutor::load(&package, limits()).unwrap();
    assert_eq!(fresh.call_i32("echo", 1), Ok(1));
}

#[test]
fn caps_initial_memory_table_and_guest_memory_growth() {
    let table = wat::parse_str("(module (table 5 funcref))").unwrap();
    assert!(matches!(
        WasmExecutor::load(&package("wasm", table), limits()),
        Err(WasmExecutorError::Instantiation(_))
    ));

    let initial = wat::parse_str("(module (memory 2))").unwrap();
    assert!(matches!(
        WasmExecutor::load(&package("wasm", initial), limits()),
        Err(WasmExecutorError::Instantiation(_))
    ));
    let bytes = wat::parse_str(
        r#"(module (memory 1)
        (func (export "grow") (param i32) (result i32) local.get 0 memory.grow))"#,
    )
    .unwrap();
    let mut executor = WasmExecutor::load(&package("wasm", bytes), limits()).unwrap();
    assert_eq!(
        executor.call_i32("grow", 1),
        Err(WasmExecutorError::GrowthDenied)
    );
}

fn message_package(offset: i32, capacity: i32, body: &str) -> PluginPackage {
    let text = format!(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const {offset}))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const {capacity}))
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) {body}))"#
    );
    package("wasm", wat::parse_str(text).unwrap())
}

#[test]
fn exchanges_owned_bytes_with_a_versioned_guest_buffer() {
    use obscur_plugin_executor_wasm::executor::WasmMessageLimits;

    let package = message_package(16, 8, "i32.const 16 i32.const 88 i32.store8 local.get 0");
    let mut instance = WasmExecutor::load(&package, limits()).unwrap();
    let budget = WasmMessageLimits {
        request_bytes: 3,
        response_bytes: 3,
    };
    assert_eq!(
        instance.exchange(b"abcd", budget),
        Err(WasmExecutorError::RequestTooLarge)
    );
    assert_eq!(instance.exchange(b"abc", budget), Ok(b"Xbc".to_vec()));
    assert_eq!(instance.exchange(b"a", budget), Ok(b"X".to_vec()));
    assert_eq!(
        instance.exchange(
            b"123456789",
            WasmMessageLimits {
                request_bytes: 9,
                ..budget
            }
        ),
        Err(WasmExecutorError::RequestTooLarge)
    );
}

#[test]
fn checks_guest_offsets_and_declared_response_length_before_copying() {
    use obscur_plugin_executor_wasm::executor::WasmMessageLimits;

    let budget = WasmMessageLimits {
        request_bytes: 8,
        response_bytes: 3,
    };
    for (offset, capacity) in [(-1, 8), (0, -1), (65535, 8)] {
        let package = message_package(offset, capacity, "unreachable");
        let mut instance = WasmExecutor::load(&package, limits()).unwrap();
        assert_eq!(
            instance.exchange(b"", budget),
            Err(WasmExecutorError::InvalidMessageBuffer)
        );
    }
    for (reply, expected) in [
        (-1, WasmExecutorError::InvalidResponseLength),
        (9, WasmExecutorError::InvalidResponseLength),
        (4, WasmExecutorError::ResponseTooLarge),
    ] {
        let package = message_package(16, 8, &format!("i32.const {reply}"));
        let mut instance = WasmExecutor::load(&package, limits()).unwrap();
        assert_eq!(instance.exchange(b"", budget), Err(expected));
        assert_eq!(
            instance.exchange(b"", budget),
            Err(WasmExecutorError::Faulted)
        );
    }
}

fn lifecycle_package(initialize: &str, deactivate: &str) -> PluginPackage {
    let text = format!(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 16))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 8))
        (global $active (mut i32) (i32.const 0))
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) {initialize})
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32)
            global.get $active i32.eqz if unreachable end
            i32.const 16 i32.const 68 i32.store8 i32.const 1)
        {deactivate})"#
    );
    package("wasm", wat::parse_str(text).unwrap())
}

#[test]
fn lifecycle_initializes_dispatches_and_tears_down_the_same_guest() {
    use obscur_plugin_executor_wasm::executor::{WasmMessageLimits, WasmPluginInstance};

    let package = lifecycle_package(
        "i32.const 1 global.set $active local.get 0",
        r#"(func (export "obscur_plugin_deactivate_v1") (param i32) (result i32)
            global.get $active i32.eqz if unreachable end
            i32.const 0 global.set $active
            i32.const 16 i32.const 83 i32.store8 i32.const 1)"#,
    );
    let budget = WasmMessageLimits {
        request_bytes: 8,
        response_bytes: 8,
    };
    let (mut guest, initialized) =
        WasmPluginInstance::activate(&package, limits(), budget, b"init").unwrap();
    assert_eq!(initialized, b"init");
    assert_eq!(guest.exchange(b"").unwrap(), b"D");
    assert_eq!(guest.deactivate(b"").unwrap(), b"S");
}

#[test]
fn lifecycle_rejects_incomplete_contracts_and_failed_initialization() {
    use obscur_plugin_executor_wasm::executor::{WasmMessageLimits, WasmPluginInstance};

    let budget = WasmMessageLimits {
        request_bytes: 8,
        response_bytes: 8,
    };
    let incomplete = lifecycle_package("unreachable", "");
    assert!(matches!(
        WasmPluginInstance::activate(&incomplete, limits(), budget, b""),
        Err(WasmExecutorError::InvalidExport)
    ));

    let teardown =
        "(func (export \"obscur_plugin_deactivate_v1\") (param i32) (result i32) local.get 0)";
    let trapping = lifecycle_package("unreachable", teardown);
    assert!(matches!(
        WasmPluginInstance::activate(&trapping, limits(), budget, b""),
        Err(WasmExecutorError::Trap(_))
    ));

    let invalid_reply = lifecycle_package("i32.const -1", teardown);
    assert!(matches!(
        WasmPluginInstance::activate(&invalid_reply, limits(), budget, b""),
        Err(WasmExecutorError::InvalidResponseLength)
    ));
}

#[test]
fn host_call_import_checks_guest_memory_and_discards_requests_on_trap() {
    let bytes = wat::parse_str(
        r#"(module
        (import "obscur_host" "submit_v1" (func $submit (param i32 i32) (result i32)))
        (memory (export "memory") 1)
        (data (i32.const 16) "data")
        (func (export "submit") (param i32) (result i32) local.get 0 i32.const 4 call $submit)
        (func (export "trap") (param i32) (result i32)
            i32.const 16 i32.const 4 call $submit drop unreachable))"#,
    )
    .unwrap();
    let package = package("wasm", bytes);
    assert!(matches!(
        WasmExecutor::load(&package, limits()),
        Err(WasmExecutorError::ImportsUnsupported)
    ));
    let mut guest = WasmExecutor::load_with_host_calls(&package, limits(), 4).unwrap();
    assert_eq!(guest.call_i32("submit", 65534), Ok(1));
    assert_eq!(guest.call_i32("submit", -1), Ok(1));
    assert!(guest.take_host_call().is_none());
    assert_eq!(guest.call_i32("submit", 16), Ok(0));
    assert_eq!(guest.take_host_call(), Some(b"data".to_vec()));
    assert!(matches!(
        guest.call_i32("trap", 0),
        Err(WasmExecutorError::Trap(_))
    ));
    assert!(guest.take_host_call().is_none());
    let mut guest = WasmExecutor::load_with_host_calls(&package, limits(), 3).unwrap();
    assert_eq!(guest.call_i32("submit", 16), Ok(1));
    assert!(guest.take_host_call().is_none());
}
