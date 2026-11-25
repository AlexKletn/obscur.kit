//! Dependency plans and diagnostics through the public host interface.

use obscur_plugin_host::{
    dependency::{DependencyConflict, DependencyResolver},
    manager::{PluginCatalog, PluginManager},
};
use obscur_plugin_model::{
    identity::PluginId,
    manifest::{
        DependencyKind,
        DependencyKind::{Optional, Required, ServiceOnly},
        DependencyManifest, Entrypoint, HostCompatibility, PluginKind, PluginManifest, Version,
        VersionReq,
    },
};

fn id(name: &str) -> PluginId {
    PluginId::new(name).unwrap()
}

#[test]
fn catalog_selects_a_version_and_host_loads_it_on_demand() {
    use obscur_plugin_api::{
        plugin::{PluginApi, PluginRegistration},
        status::PluginApiError,
    };
    use obscur_plugin_model::{
        contribution::{ExtensionPoint, ViewContribution},
        identity::ContributionId,
    };
    use obscur_routing::router::RouteRegistry;
    use obscur_view::view::{View, ViewNode};
    use std::time::{Duration, Instant};

    struct TestPlugin(ExtensionPoint, ContributionId);
    impl PluginApi for TestPlugin {
        fn initialize(&mut self, staged: &mut PluginRegistration) -> Result<(), PluginApiError> {
            staged.contribute_view(
                self.0.clone(),
                ViewContribution::new(self.1.clone(), "Test"),
                || View::single(ViewNode::text("ready")),
            );
            Ok(())
        }
    }

    let mut catalog = PluginCatalog::default();
    let first = plugin("example.plugin", &[]);
    let mut second = first.clone();
    second.version = Version::new(2, 0, 0);
    assert!(catalog.insert(first.clone()));
    assert!(catalog.insert(second.clone()));
    assert!(!catalog.insert(first.clone()));
    let accepted = VersionReq::parse("^1").unwrap();
    let selected = catalog.select(&first.id, &accepted).unwrap();
    assert_eq!(selected, &first);
    let mut manager = PluginManager::new(RouteRegistry::new());
    let point = ExtensionPoint::new("app.views").unwrap();
    let view = ContributionId::new("test").unwrap();
    manager.publish(point.clone());
    assert!(manager.state(&first.id).is_none());
    let opened = manager
        .open_manifest_view(selected, &point, &view, |manifest| {
            assert_eq!(manifest, &first);
            Ok(TestPlugin(point.clone(), view.clone()))
        })
        .unwrap();
    assert!(opened.is_some());
    manager.suspend(&first.id).unwrap();
    assert!(
        manager
            .open_manifest_view(selected, &point, &view, |_| -> Result<TestPlugin, _> {
                panic!("suspended plugin must be reused")
            })
            .unwrap()
            .is_some()
    );
    assert!(matches!(
        manager.open_manifest_view(&second, &point, &view, |_| -> Result<TestPlugin, _> {
            panic!("opening another version must not replace the active instance")
        }),
        Err(PluginApiError::Rejected(_))
    ));
    assert_eq!(
        manager.activate_manifest(selected, |_| -> Result<TestPlugin, _> {
            panic!("active plugin must not be loaded twice")
        }),
        Err(PluginApiError::AlreadyActive)
    );
    let before_suspend = Instant::now();
    let idle = Duration::from_secs(60);
    manager.suspend(&first.id).unwrap();
    assert!(manager.unload_idle(before_suspend, idle).is_empty());
    assert_eq!(
        manager.unload_idle(Instant::now() + idle, idle),
        vec![(first.id.clone(), Ok(()))]
    );
    assert!(manager.unload_idle(Instant::now() + idle, idle).is_empty());
    assert!(
        manager
            .open_manifest_view(selected, &point, &view, |_| {
                Ok(TestPlugin(point.clone(), view.clone()))
            })
            .unwrap()
            .is_some()
    );
    assert_eq!(catalog.remove(&second.id, &second.version), Some(second));
    assert_eq!(catalog.manifests().count(), 1);
}

fn plugin(name: &str, dependencies: &[(&str, DependencyKind)]) -> PluginManifest {
    PluginManifest {
        schema_version: 1,
        id: id(name),
        name: name.into(),
        version: Version::new(1, 0, 0),
        author: "Test".into(),
        kind: PluginKind::Wasm,
        compatibility: HostCompatibility {
            plugin_api: VersionReq::STAR,
            ui_ir: VersionReq::STAR,
            applications: Default::default(),
            required_services: Default::default(),
            optional_features: Default::default(),
            extension_contracts: Default::default(),
        },
        entrypoint: Entrypoint {
            path: "plugin.wasm".into(),
        },
        contributions: Default::default(),
        resources: vec![],
        css: vec![],
        themes: vec![],
        localization: vec![],
        capabilities: vec![],
        platforms: vec![],
        dependencies: dependencies
            .iter()
            .map(|&(provider, kind)| DependencyManifest {
                id: id(provider),
                version: VersionReq::parse("^1").unwrap(),
                kind,
            })
            .collect(),
        files: vec![],
    }
}

#[test]
fn plans_shared_providers_first_and_ignores_absent_optional_providers() {
    let app = plugin(
        "app",
        &[
            ("ui", Required),
            ("absent", Optional),
            ("network", ServiceOnly),
        ],
    );
    let ui = plugin("ui", &[("network", Required)]);
    let network = plugin("network", &[]);
    for inventory in [[&app, &ui, &network], [&network, &app, &ui]] {
        let graph = DependencyResolver::resolve(inventory).unwrap();
        assert_eq!(graph.activation_order(), [&network.id, &ui.id, &app.id]);
        let edges: Vec<_> = graph
            .dependencies(&app.id)
            .unwrap()
            .map(|edge| (edge.id.as_str(), edge.kind))
            .collect();
        assert_eq!(edges, [("ui", Required), ("network", ServiceOnly)]);
    }
}

#[test]
fn reports_closed_cycles_without_unrelated_consumers() {
    let app = plugin("app", &[("backend", Required)]);
    let backend = plugin("backend", &[("storage", ServiceOnly)]);
    let storage = plugin("storage", &[("backend", Optional)]);
    let result = DependencyResolver::resolve([&app, &backend, &storage]);
    assert!(matches!(result, Err(DependencyConflict::Cycle(path))
        if path.iter().eq([&backend.id, &storage.id, &backend.id])));

    let circular = plugin("circular", &[("circular", Optional)]);
    assert!(matches!(DependencyResolver::resolve([&circular]),
        Err(DependencyConflict::Cycle(path)) if path == [id("circular"), id("circular")]));
}

#[test]
fn reports_missing_providers_and_repeated_declarations() {
    let app = plugin("app", &[("service", ServiceOnly)]);
    assert!(
        matches!(DependencyResolver::resolve([&app]), Err(DependencyConflict::MissingProvider {
        consumer, dependency
    }) if consumer == app.id && dependency == app.dependencies[0])
    );

    let service = plugin("service", &[]);
    assert!(matches!(DependencyResolver::resolve([&service, &service]),
        Err(DependencyConflict::DuplicatePlugin(plugin)) if plugin == service.id));

    let repeated = plugin("app", &[("service", Required), ("service", Optional)]);
    assert!(matches!(DependencyResolver::resolve([&repeated, &service]),
        Err(DependencyConflict::DuplicateDependency { consumer, provider })
        if consumer == repeated.id && provider == service.id));
}

#[test]
fn checks_provider_updates_against_existing_consumers() {
    let consumer = plugin("app", &[("service", Optional)]);
    let service = plugin("service", &[]);
    let graph = DependencyResolver::resolve([&consumer, &service]).unwrap();

    let mut proposed = service.clone();
    proposed.version = Version::new(2, 0, 0);
    assert!(
        matches!(graph.with_update(&proposed), Err(DependencyConflict::IncompatibleVersion {
        consumer: found_consumer, dependency, found,
    }) if found_consumer == consumer.id && dependency == consumer.dependencies[0] && found == proposed.version)
    );
    assert_eq!(graph.activation_order(), [&service.id, &consumer.id]);

    proposed.version = Version::new(1, 1, 0);
    let updated = graph.with_update(&proposed).unwrap();
    assert!(std::ptr::eq(updated.activation_order()[0], &proposed.id));
}
