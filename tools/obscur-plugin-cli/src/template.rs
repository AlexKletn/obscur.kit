//! Creates starter plugin sources and manifests for the CLI pack command.

use std::{error::Error, fs, path::Path};

use obscur_plugin_model::identity::PluginId;
use serde_json::json;

const WASM_GUEST: &str = include_str!("template.doc/guest.wat");
const PROCESS_GUEST: &str = include_str!("template.doc/process_guest.rs");
const NATIVE_GUEST: &str = include_str!("template.doc/native_guest.rs");

pub(crate) fn wasm(directory: &Path, id: &PluginId) -> Result<(), Box<dyn Error>> {
    let wasm = wat::parse_str(WASM_GUEST)?;
    let manifest = manifest(id, "wasm", "guest.wasm")?;
    fs::create_dir(directory)?;
    fs::write(directory.join("guest.wat"), WASM_GUEST)?;
    fs::write(directory.join("guest.wasm"), wasm)?;
    fs::write(directory.join("manifest.json"), manifest)?;
    Ok(())
}

pub(crate) fn process(directory: &Path, id: &PluginId) -> Result<(), Box<dyn Error>> {
    let executable = format!("guest{}", std::env::consts::EXE_SUFFIX);
    let manifest = manifest(id, "process", &executable)?;
    fs::create_dir(directory)?;
    fs::write(directory.join("guest.rs"), PROCESS_GUEST)?;
    fs::write(directory.join("manifest.json"), manifest)?;
    fs::write(
        directory.join("README.md"),
        format!(
            "# Process plugin\n\nBuild with `rustc --edition=2024 guest.rs -o {executable}`. Then run `obscur-plugin-cli pack` on this directory.\n\nThe host passes an issued service handle as the only argument. The guest calls its `get` operation with JSON null, reads the framed reply, and writes the result to stderr.\n"
        ),
    )?;
    Ok(())
}

pub(crate) fn native(directory: &Path, id: &PluginId, sdk: &Path) -> Result<(), Box<dyn Error>> {
    let sdk = sdk.canonicalize()?;
    if !sdk.join("Cargo.toml").is_file() {
        return Err("SDK directory must contain Cargo.toml".into());
    }
    let sdk = sdk.to_str().ok_or("SDK directory must be UTF-8")?;
    let sdk = serde_json::to_string(sdk)?;
    let library = format!(
        "{}guest{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let entrypoint = format!("target/release/{library}");
    let manifest = manifest(id, "native", &entrypoint)?;
    let cargo = format!(
        "[workspace]\n\n[package]\nname = \"native-guest\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"guest\"\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nobscur-plugin-sdk = {{ path = {sdk} }}\n"
    );
    fs::create_dir(directory)?;
    fs::create_dir(directory.join("src"))?;
    fs::write(directory.join("src/lib.rs"), NATIVE_GUEST)?;
    fs::write(directory.join("manifest.json"), manifest)?;
    fs::write(directory.join("Cargo.toml"), cargo)?;
    fs::write(
        directory.join("README.md"),
        "# Native plugin\n\nBuild with `cargo build --release`. Then run `obscur-plugin-cli pack` on this directory. The library exports the version-1 native lifecycle table and echoes dispatch requests.\n",
    )?;
    Ok(())
}

fn manifest(id: &PluginId, kind: &str, entrypoint: &str) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec_pretty(&json!({
        "schema_version": 1,
        "id": id.as_str(),
        "name": id.as_str(),
        "author": "Plugin author",
        "version": "0.1.0",
        "kind": kind,
        "compatibility": {"plugin_api": "^1", "ui_ir": "^1"},
        "entrypoint": {"path": entrypoint},
        "files": [{"path": entrypoint}],
    }))
}
