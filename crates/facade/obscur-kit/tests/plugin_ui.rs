//! Shared UI IR and event contract for sandboxed plugin contributions.

use std::{cell::RefCell, rc::Rc};

use obscur_kit::{
    geometry::Size,
    identity::{InternedSymbol, RuntimeId},
    plugins::{
        ExtensionPoint, PluginApi, PluginApiError, PluginId, PluginManager, PluginRegistration,
        PluginUi,
    },
    runtime::UiRuntime,
};
use obscur_routing::router::RouteRegistry;
use obscur_ui_ir::{
    codec::UiIrEncoder,
    document::{UiIrDocument, UiIrEvent, UiIrNode, UiIrProperty},
};
use serde_json::json;

fn envelope(node: UiIrNode) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "version": 1,
        "views": [{
            "point": "workspace.tools",
            "id": "demo",
            "title": "Demo",
            "ir": UiIrEncoder::new().encode(&UiIrDocument::new(node))
        }]
    }))
    .unwrap()
}

struct UiPlugin(Rc<PluginUi>);

impl PluginApi for UiPlugin {
    fn initialize(&mut self, staged: &mut PluginRegistration) -> Result<(), PluginApiError> {
        self.0.stage(staged);
        Ok(())
    }
}

#[test]
fn guest_contract_mismatch_skips_only_the_incompatible_view() {
    use obscur_kit::plugins::Version;

    let bytes = envelope(UiIrNode::text(1, "View"));
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let views = wire["views"].as_array_mut().unwrap();
    views[0]["contract"] = json!("^2");
    let mut unsupported = views[0].clone();
    unsupported["id"] = json!("unsupported");
    unsupported["contract"] = json!("^3");
    views.push(unsupported);

    let bytes = serde_json::to_vec(&wire).unwrap();
    let ui = Rc::new(PluginUi::decode(&bytes, 4096, 4, || InternedSymbol::from_raw(1)).unwrap());
    let point = ExtensionPoint::new("workspace.tools").unwrap();
    let id = PluginId::new("example").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    assert!(
        manager.publish_versioned(point.clone(), Version::new(2, 1, 0), |_, view| {
            assert_eq!(view.id().as_str(), "demo");
            Ok(None)
        })
    );
    manager.activate(id, UiPlugin(ui)).unwrap();
    let mounted: Vec<_> = manager.contributions().views_at(&point).collect();
    assert_eq!(mounted.len(), 1);
    assert_eq!(mounted[0].definition().id().as_str(), "demo");
}

#[test]
fn guest_view_mounts_in_a_host_point_and_uses_host_assigned_event_symbols() {
    let button = UiIrNode::element(1, "button")
        .with_event(UiIrEvent::new("click", "apply"))
        .with_child(UiIrNode::text(2, "Apply"));
    let symbol = InternedSymbol::from_raw(71);
    let ui = Rc::new(PluginUi::decode(&envelope(button), 4096, 4, || symbol).unwrap());

    let point = ExtensionPoint::new("workspace.tools").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    manager.publish(point.clone());
    let id = PluginId::new("example").unwrap();
    manager
        .activate(id.clone(), UiPlugin(Rc::clone(&ui)))
        .unwrap();
    let view = manager
        .contributions()
        .views_at(&point)
        .next()
        .unwrap()
        .render()
        .unwrap();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(320.0, 200.0));
    runtime.render_now(&view, &[]);
    assert!(
        runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("Apply"))
    );

    let emitted = Rc::new(RefCell::new(None));
    for command in ui.commands() {
        let ui = Rc::clone(&ui);
        let emitted = Rc::clone(&emitted);
        runtime.on_command(command, move || {
            emitted.replace(ui.event_message(command).unwrap());
        });
    }
    assert!(runtime.dispatch_command(symbol));
    let event: serde_json::Value =
        serde_json::from_slice(emitted.borrow().as_ref().unwrap()).unwrap();
    assert_eq!(
        event,
        json!({
            "version": 1, "type": "ui_event",
            "event": {
                "point": "workspace.tools", "view": "demo", "node": 1, "event": "click", "action": "apply"
            }
        })
    );
    manager.deactivate(&id).unwrap();
    assert!(manager.contributions().views_at(&point).next().is_none());
}

#[test]
fn invalid_guest_ui_is_rejected_before_staging() {
    let symbol = InternedSymbol::from_raw(71);
    let invalid =
        UiIrNode::element(1, "button").with_property(UiIrProperty::text("src", "file:///private"));
    assert!(PluginUi::decode(&envelope(invalid), 4096, 16, || symbol).is_err());

    let button = || UiIrNode::element(2, "button").with_event(UiIrEvent::new("click", "run"));
    let duplicate = UiIrNode::fragment(1)
        .with_child(button())
        .with_child(UiIrNode::element(3, "button").with_event(UiIrEvent::new("click", "another")));
    assert!(PluginUi::decode(&envelope(duplicate), 4096, 16, || symbol).is_err());

    let deep = (1..70).fold(UiIrNode::text(0, "leaf"), |node, id| {
        UiIrNode::fragment(id).with_child(node)
    });
    assert!(PluginUi::decode(&envelope(deep), 32768, 128, || symbol).is_err());
    let two_nodes = UiIrNode::element(1, "div").with_child(UiIrNode::text(2, "leaf"));
    assert!(PluginUi::decode(&envelope(two_nodes), 4096, 1, || symbol).is_err());
}

#[cfg(feature = "plugin-wasm")]
#[test]
fn managed_wasm_returns_its_ui_and_receives_the_runtime_event() {
    use obscur_kit::plugins::{
        ManagedWasmPlugin, PackageEntry, PackageError, PackageLimits, PackagePath, PackageReader,
        PackageValidator, Version, WasmLimits, WasmMessageLimits,
    };
    use sha2::{Digest, Sha256};

    struct Entries(Vec<PackageEntry>);
    impl PackageReader for Entries {
        fn read_entries(self, _: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
            Ok(self.0)
        }
    }

    let ui = envelope(
        UiIrNode::element(1, "button")
            .with_event(UiIrEvent::new("click", "apply"))
            .with_child(UiIrNode::text(2, "Apply")),
    );
    use std::fmt::Write;
    let encoded = ui.iter().fold(String::new(), |mut text, byte| {
        write!(text, "\\{byte:02x}").unwrap();
        text
    });
    let wasm = wat::parse_str(format!(
        r#"(module
        (memory (export "memory") 1)
        (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 0))
        (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 4096))
        (data (i32.const 0) "{encoded}")
        (func (export "obscur_plugin_initialize_v1") (param i32) (result i32) i32.const {})
        (func (export "obscur_plugin_dispatch_v1") (param i32) (result i32) local.get 0)
        (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32) i32.const 0)
    )"#,
        ui.len()
    ))
    .unwrap();
    let path = "module.wasm";
    let manifest = json!({
        "schema_version": 1, "id": "workspace.guest", "name": "Guest", "author": "Example",
        "version": "1.0.0", "kind": "wasm", "compatibility": {
            "plugin_api": "^1", "ui_ir": "^1",
            "extension_contracts": {"workspace.tools": "^2", "workspace.blocked": "^3"}
        },
        "entrypoint": {"path": path},
        "files": [{"path": path, "sha256": format!("{:x}", Sha256::digest(&wasm))}]
    });
    let package = PackageValidator::new(Version::new(1, 0, 0), Version::new(1, 0, 0))
        .load(Entries(vec![
            PackageEntry::file(PackagePath::parse(path).unwrap(), wasm),
            PackageEntry::file(
                PackagePath::parse("manifest.json").unwrap(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
        ]))
        .unwrap();

    let decoded = Rc::new(RefCell::new(None));
    let command = InternedSymbol::from_raw(91);
    let registered = Rc::clone(&decoded);
    let plugin = ManagedWasmPlugin::new(
        package,
        WasmLimits {
            module_bytes: 4096,
            memory_bytes: 65536,
            table_elements: 0,
            fuel_per_call: 1000,
        },
        WasmMessageLimits {
            request_bytes: 4096,
            response_bytes: 4096,
        },
        Vec::new(),
        move |bytes, staged| {
            let ui = PluginUi::decode(bytes, 4096, 8, || command)?;
            ui.stage(staged);
            staged.contribute_view(
                ExtensionPoint::new("workspace.blocked").unwrap(),
                obscur_kit::plugins::ViewContribution::new(
                    obscur_kit::plugins::ContributionId::new("other").unwrap(),
                    "Other",
                )
                .with_contract(obscur_kit::plugins::VersionReq::parse("^2").unwrap()),
                || panic!("incompatible view must not render"),
            );
            registered.replace(Some(ui));
            Ok(())
        },
    );
    let handle = plugin.handle();
    let point = ExtensionPoint::new("workspace.tools").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    manager.publish_versioned(point.clone(), Version::new(2, 0, 0), |_, _| Ok(None));
    let blocked = ExtensionPoint::new("workspace.blocked").unwrap();
    manager.publish_versioned(blocked.clone(), Version::new(2, 0, 0), |_, _| {
        panic!("manifest-incompatible view must not mount")
    });
    manager.activate_wasm(plugin).unwrap();
    assert!(manager.contributions().views_at(&blocked).next().is_none());

    let view = manager
        .contributions()
        .views_at(&point)
        .next()
        .unwrap()
        .render()
        .unwrap();
    let [obscur_kit::view::view::ViewNode::Element { events, .. }] = view.root().nodes() else {
        panic!("expected the guest button");
    };
    assert_eq!(events[0].command(), command);
    let ui = decoded.take().unwrap();
    let reply = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&reply);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(320.0, 200.0));
    runtime.on_command(command, move || {
        let event = ui.event_message(command).unwrap().unwrap();
        output.replace(handle.exchange(&event).unwrap());
    });
    runtime.render_now(&view, &[]);
    assert!(runtime.dispatch_command(command));
    let event: serde_json::Value = serde_json::from_slice(&reply.borrow()).unwrap();
    assert_eq!(event["event"]["action"], "apply");
}

#[test]
fn newer_snapshots_update_mounted_views_and_event_bindings_together() {
    fn update(revision: u64, title: &str, node: UiIrNode) -> Vec<u8> {
        let mut message: serde_json::Value = serde_json::from_slice(&envelope(node)).unwrap();
        message["revision"] = json!(revision);
        message["views"][0]["title"] = json!(title);
        serde_json::to_vec(&message).unwrap()
    }
    fn button(label: &str, action: &str) -> UiIrNode {
        UiIrNode::element(1, "button")
            .with_event(UiIrEvent::new("click", action))
            .with_child(UiIrNode::text(2, label))
    }

    let old_command = InternedSymbol::from_raw(71);
    let new_command = InternedSymbol::from_raw(72);
    let ui = Rc::new(
        PluginUi::decode(&envelope(button("Before", "apply")), 4096, 4, || {
            old_command
        })
        .unwrap(),
    );
    let point = ExtensionPoint::new("workspace.tools").unwrap();
    let mut manager = PluginManager::new(RouteRegistry::new());
    manager.publish(point.clone());
    let id = PluginId::new("example").unwrap();
    manager
        .activate(id.clone(), UiPlugin(Rc::clone(&ui)))
        .unwrap();
    let view = manager.contributions().views_at(&point).next().unwrap();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(320.0, 200.0));
    runtime.render_now(&view.render().unwrap(), &[]);
    assert!(
        runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("Before"))
    );

    assert!(
        ui.apply_update(&update(1, "Demo", button("After", "apply")), || panic!(
            "same binding must keep its command"
        ))
        .unwrap()
    );
    assert_eq!(ui.commands().collect::<Vec<_>>(), [old_command]);
    runtime.render_now(&view.render().unwrap(), &[]);
    assert!(
        runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("After"))
    );
    assert!(
        !runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("Before"))
    );
    assert!(
        !ui.apply_update(&update(1, "Demo", button("Stale", "apply")), || old_command)
            .unwrap()
    );

    assert!(
        ui.apply_update(
            &update(2, "Changed title", button("Rejected", "other")),
            || new_command
        )
        .is_err()
    );
    assert!(
        ui.apply_update(&update(2, "Demo", button("Collision", "other")), || {
            old_command
        })
        .is_err()
    );
    assert_eq!(ui.revision(), 1);
    assert!(ui.event_message(old_command).unwrap().is_some());
    runtime.on_command(old_command, || {});
    assert!(runtime.has_command_handler(old_command));

    assert!(
        ui.apply_update(&update(3, "Demo", button("Final", "other")), || new_command)
            .unwrap()
    );
    assert_eq!(ui.revision(), 3);
    assert!(ui.event_message(old_command).unwrap().is_none());
    assert!(runtime.remove_command_handler(old_command));
    assert!(!runtime.has_command_handler(old_command));
    let bytes = ui.event_message(new_command).unwrap().unwrap();
    let event: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(event["event"]["action"], "other");
}
