//! Pack and execute a minimal guest through the CLI.

use std::{fs, process::Command};

use serde_json::{Value, json};

#[test]
fn executes_both_lifecycle_hooks_in_a_validated_archive() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir(&source).unwrap();
    let wasm = wat::parse_str(
        r#"(module
          (memory (export "memory") 1)
          (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 0))
          (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 16))
          (func (export "obscur_plugin_initialize_v1") (param i32) (result i32)
            local.get 0)
          (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
          (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32)
            i32.const 0 i32.const 43 i32.store8 i32.const 1))"#,
    )
    .unwrap();
    fs::write(source.join("guest.wasm"), wasm).unwrap();
    fs::write(
        source.join("manifest.json"),
        json!({
            "schema_version": 1, "id": "example.smoke", "name": "Smoke", "author": "Fixture",
            "version": "1.0.0", "kind": "wasm",
            "compatibility": {"plugin_api": "^1", "ui_ir": "^1"},
            "entrypoint": {"path": "guest.wasm"},
            "files": [{"path": "guest.wasm"}]
        })
        .to_string(),
    )
    .unwrap();
    let archive = temporary.path().join("guest.zip");
    let cli = env!("CARGO_BIN_EXE_obscur-plugin-cli");
    let packed = Command::new(cli)
        .arg("pack")
        .arg(&source)
        .arg(&archive)
        .args(["1.0.0", "1.0.0"])
        .output()
        .unwrap();
    assert!(packed.status.success(), "{:?}", packed.stderr);
    let initial = temporary.path().join("initial.bin");
    let dispatch = temporary.path().join("dispatch.bin");
    fs::write(&initial, [42]).unwrap();
    fs::write(&dispatch, [1, 2, 3]).unwrap();
    let result = Command::new(cli)
        .arg("smoke-wasm")
        .arg(archive)
        .args(["1.0.0", "1.0.0"])
        .arg("--initial")
        .arg(initial)
        .arg("--dispatch")
        .arg(dispatch)
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["id"], "example.smoke");
    assert_eq!(report["initialize_reply"], json!([42]));
    assert_eq!(report["dispatch_reply"], json!([1, 2, 3]));
    assert_eq!(report["deactivate_reply"], json!([43]));

    let wasm = wat::parse_str(
        r#"(module
          (import "obscur_host" "submit_v1" (func $submit (param i32 i32) (result i32)))
          (memory (export "memory") 1)
          (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 0))
          (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 64))
          (data (i32.const 128) "\00\00\00\12\01\00\00\00\00\00\00\00\00\00\03getnull")
          (func (export "obscur_plugin_initialize_v1") (param i32) (result i32)
            i32.const 133 i32.const 0 i64.load i64.store
            i32.const 128 i32.const 22 call $submit drop i32.const 0)
          (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
          (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0))"#,
    )
    .unwrap();
    fs::write(source.join("guest.wasm"), wasm).unwrap();
    let archive = temporary.path().join("mock.zip");
    let fixture = temporary.path().join("response.json");
    fs::write(&fixture, b"true").unwrap();
    let packed = Command::new(cli)
        .arg("pack")
        .arg(&source)
        .arg(&archive)
        .args(["1.0.0", "1.0.0"])
        .output()
        .unwrap();
    assert!(packed.status.success(), "{:?}", packed.stderr);
    let result = Command::new(cli)
        .arg("smoke-wasm")
        .arg(archive)
        .args(["1.0.0", "1.0.0", "--mock-response"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        report["initialize_reply"],
        json!([0, 0, 0, 6, 1, 0, 116, 114, 117, 101])
    );
}
