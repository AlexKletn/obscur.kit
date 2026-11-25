//! Command-line validation and contract inspection for plugin packages.

mod inspect_ui;
mod pack;
mod smoke_wasm;
mod template;

use std::{
    error::Error,
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use obscur_plugin_model::{identity::PluginId, manifest::Version};
use obscur_plugin_package::{package::PluginPackage, validation::PackageValidator};
use serde_json::json;

const USAGE: &str = "Usage: obscur-plugin-cli <validate|inspect-contracts|smoke-wasm> <archive> <plugin-api-version> <ui-ir-version> [options]
       obscur-plugin-cli pack <source-directory> <archive.tar|archive.zip> <plugin-api-version> <ui-ir-version> [options]
       obscur-plugin-cli inspect-ui <document.okir>
       obscur-plugin-cli new-wasm <directory> <plugin-id>
       obscur-plugin-cli new-process <directory> <plugin-id>
       obscur-plugin-cli new-native <directory> <plugin-id> <sdk-directory>
Archive formats: .tar, .tar.gz, .tar.zst, .zip
Options:
  --application <id> <version>
  --service <id> <version>
  --feature <id>
  --allow-capability <id>
  --provider <plugin-id> <version>
  --platform <os> <arch>
Smoke options:
  --initial <file>
  --dispatch <file>
  --mock-response <json-file>";

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1), io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("obscur-plugin-cli: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    mut args: impl Iterator<Item = OsString>,
    mut output: impl Write,
) -> Result<(), Box<dyn Error>> {
    let command = args.next();
    if command
        .as_deref()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        writeln!(output, "{USAGE}")?;
        return Ok(());
    }
    if command.as_deref().is_some_and(|arg| arg == "pack") {
        let source = args.next().ok_or("missing source directory")?;
        let destination = args.next().ok_or("missing output archive")?;
        let api = arg(&mut args, "plugin API version")?.parse()?;
        let ui_ir = arg(&mut args, "UI IR version")?.parse()?;
        let validator = configure(PackageValidator::new(api, ui_ir), &mut args, None)?;
        pack::build(Path::new(&source), Path::new(&destination), &validator)?;
        return Ok(());
    }
    if command.as_deref().is_some_and(|arg| arg == "inspect-ui") {
        let document = args.next().ok_or("missing OKIR document")?;
        if args.next().is_some() {
            return Err("inspect-ui accepts one document".into());
        }
        return inspect_ui::run(Path::new(&document), output);
    }
    if let Some(kind @ ("new-wasm" | "new-process" | "new-native")) =
        command.as_deref().and_then(|arg| arg.to_str())
    {
        let directory = args.next().ok_or("missing template directory")?;
        let id = arg(&mut args, "plugin ID")?;
        let sdk = if kind == "new-native" {
            Some(args.next().ok_or("missing SDK directory")?)
        } else {
            None
        };
        if args.next().is_some() {
            return Err("unexpected template argument".into());
        }
        let id = PluginId::new(&id).ok_or("invalid plugin ID")?;
        return match (kind, sdk) {
            (_, Some(sdk)) => template::native(Path::new(&directory), &id, Path::new(&sdk)),
            ("new-wasm", _) => template::wasm(Path::new(&directory), &id),
            _ => template::process(Path::new(&directory), &id),
        };
    }
    let command = match command.as_deref().and_then(|arg| arg.to_str()) {
        Some(command @ ("validate" | "inspect-contracts" | "smoke-wasm")) => command,
        _ => return Err(USAGE.into()),
    };
    let archive = args.next().ok_or("missing package archive")?;
    let api = arg(&mut args, "plugin API version")?.parse()?;
    let ui_ir = arg(&mut args, "UI IR version")?.parse()?;
    let mut requests = (command == "smoke-wasm").then(smoke_wasm::Requests::default);
    let validator = configure(
        PackageValidator::new(api, ui_ir),
        &mut args,
        requests.as_mut(),
    )?;
    let package = validator.load_path(Path::new(&archive))?;
    if let Some(requests) = requests {
        return smoke_wasm::run(&package, &requests, output);
    }
    match command {
        "inspect-contracts" => inspect_contracts(&package, output)?,
        _ => writeln!(
            output,
            "{} {}",
            package.manifest().id.as_str(),
            package.manifest().version
        )?,
    }
    Ok(())
}

fn configure(
    mut validator: PackageValidator,
    args: &mut impl Iterator<Item = OsString>,
    mut smoke: Option<&mut smoke_wasm::Requests>,
) -> Result<PackageValidator, Box<dyn Error>> {
    while let Some(option) = args.next() {
        if matches!(
            option.to_str(),
            Some("--initial" | "--dispatch" | "--mock-response")
        ) {
            let smoke = smoke
                .as_deref_mut()
                .ok_or("request files require smoke-wasm")?;
            let path = PathBuf::from(args.next().ok_or("missing WASM request file")?);
            let slot = match option.to_str() {
                Some("--initial") => &mut smoke.initial,
                Some("--dispatch") => &mut smoke.dispatch,
                _ => &mut smoke.mock_response,
            };
            if slot.replace(path).is_some() {
                return Err(format!("repeated option {option:?}").into());
            }
            continue;
        }
        validator = match option.to_str() {
            Some("--application") => validator.with_application(
                arg(args, "application ID")?,
                arg(args, "application version")?.parse()?,
            ),
            Some("--service") => validator.with_host_service(
                arg(args, "service ID")?,
                arg(args, "service version")?.parse()?,
            ),
            Some("--feature") => validator.with_host_feature(arg(args, "feature ID")?),
            Some("--allow-capability") => validator.allow_capability(arg(args, "capability ID")?),
            Some("--provider") => {
                let id = arg(args, "provider plugin ID")?;
                let id = PluginId::new(&id).ok_or("invalid provider plugin ID")?;
                validator.with_provider(id, arg(args, "provider version")?.parse::<Version>()?)
            }
            Some("--platform") => {
                validator.with_platform(arg(args, "OS")?, arg(args, "architecture")?)
            }
            _ => return Err(format!("unknown option {option:?}").into()),
        };
    }
    Ok(validator)
}

fn arg(args: &mut impl Iterator<Item = OsString>, label: &str) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing {label}"))?
        .into_string()
        .map_err(|_| format!("{label} must be UTF-8").into())
}

fn inspect_contracts(package: &PluginPackage, mut output: impl Write) -> io::Result<()> {
    let manifest = package.manifest();
    let compatibility = &manifest.compatibility;
    let report = json!({
        "id": manifest.id.as_str(),
        "version": manifest.version,
        "plugin_api": compatibility.plugin_api,
        "ui_ir": compatibility.ui_ir,
        "applications": compatibility.applications,
        "required_services": compatibility.required_services,
        "extension_contracts": compatibility.extension_contracts,
        "optional_features": compatibility.optional_features,
        "available_host_features": package.available_host_features().collect::<Vec<_>>(),
        "capabilities": manifest.capabilities,
    });
    serde_json::to_writer_pretty(&mut output, &report)?;
    writeln!(output)
}
