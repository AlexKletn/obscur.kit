#![doc = include_str!("plugin_settings.doc/_module.md")]

use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::plugins::{
    ContributionId, ExtensionPoint, PluginApi, PluginApiError, PluginId, PluginLifecycleState,
    PluginManager, PluginRegistration, ViewContribution,
};
use obscur_kit::routing::{
    route::{Route, RoutePattern},
    router::RouteRegistry,
};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::view::view::{View, ViewNode};

fn point() -> ExtensionPoint {
    ExtensionPoint::new("app.views").unwrap()
}

fn publish_point(host: &mut PluginManager) {
    assert!(host.publish_with(point(), |plugin, view| {
        Ok(Some(Route::new(format!(
            "/integrations/{}/{}",
            plugin.as_str(),
            view.id().as_str()
        ))))
    }));
}

struct Settings {
    pages: &'static [&'static str],
    fail_initialize: bool,
    fail_deactivate: bool,
}

impl Settings {
    fn new(pages: &'static [&'static str]) -> Self {
        Self {
            pages,
            fail_initialize: false,
            fail_deactivate: false,
        }
    }
}

impl PluginApi for Settings {
    fn initialize(&mut self, registration: &mut PluginRegistration) -> Result<(), PluginApiError> {
        for &page in self.pages {
            let id = ContributionId::new(page).unwrap();
            registration.contribute_view(point(), ViewContribution::new(id, page), move || {
                View::single(ViewNode::text(page))
            });
        }
        if self.fail_initialize {
            return Err(PluginApiError::Rejected("initializer failed".into()));
        }
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        if self.fail_deactivate {
            return Err(PluginApiError::Rejected("teardown failed".into()));
        }
        Ok(())
    }
}

fn manager() -> PluginManager {
    let routes = RouteRegistry::new().with_route(RoutePattern::parse("/settings"), "settings");
    PluginManager::new(routes)
}

#[test]
fn trusted_settings_pages_join_navigation_and_leave_the_retained_tree_on_deactivation() {
    let plugin = PluginId::new("example.settings").unwrap();
    let mut host = manager();
    publish_point(&mut host);
    assert!(host.navigate("/settings").is_matched());
    host.activate(plugin.clone(), Settings::new(&["Display", "About"]))
        .unwrap();
    assert_eq!(host.state(&plugin), Some(PluginLifecycleState::Active));
    let pages = host.contributions().views();
    assert_eq!(pages[0].global_id(), "app.views/example.settings/About");
    let page_path = pages[1].route().unwrap().path().to_owned();
    assert_eq!(
        host.navigate(&format!("{page_path}?mode=test")).target(),
        Some("app.views/example.settings/Display")
    );

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let view = host.current_view().unwrap().unwrap();
    runtime.render_now(&view, &[]);
    assert!(
        runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("Display"))
    );

    host.deactivate(&plugin).unwrap();
    assert_eq!(host.state(&plugin), Some(PluginLifecycleState::Deactivated));
    assert!(host.contributions().views().is_empty());
    assert!(host.current_view().unwrap().is_none());
    assert!(!host.navigate(&page_path).is_matched());
    assert_eq!(host.back().unwrap().target(), Some("settings"));
    assert!(!host.forward().unwrap().is_matched());

    runtime.render_now(&View::empty(), &[]);
    assert!(
        !runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("Display"))
    );
    host.activate(plugin.clone(), Settings::new(&["Display"]))
        .unwrap();
    assert_eq!(host.state(&plugin), Some(PluginLifecycleState::Active));
    assert!(host.current_view().unwrap().is_some());
}

#[test]
fn failed_activation_leaves_no_partial_routes_or_pages() {
    let plugin = PluginId::new("example").unwrap();
    assert!(PluginId::new("../example").is_none());
    assert!(ContributionId::new("a/b").is_none());

    let routes = RouteRegistry::new().with_route(
        RoutePattern::parse("/integrations/example/Reserved"),
        "reserved",
    );
    let mut host = PluginManager::new(routes);
    assert_eq!(
        host.activate(plugin.clone(), Settings::new(&["First"])),
        Err(PluginApiError::ExtensionUnavailable)
    );
    publish_point(&mut host);
    assert_eq!(
        host.activate(plugin.clone(), Settings::new(&["First", "First"])),
        Err(PluginApiError::DuplicateContribution(
            "app.views/example/First".into()
        ))
    );
    assert_eq!(
        host.activate(plugin.clone(), Settings::new(&["First", "Reserved"])),
        Err(PluginApiError::RouteConflict(
            "/integrations/example/Reserved".into()
        ))
    );
    let mut rejected = Settings::new(&["First"]);
    rejected.fail_initialize = true;
    assert_eq!(
        host.activate(plugin.clone(), rejected),
        Err(PluginApiError::Rejected("initializer failed".into()))
    );
    assert_eq!(host.state(&plugin), None);
    assert!(host.contributions().views().is_empty());
    assert!(!host.navigate("/integrations/example/First").is_matched());
    assert_eq!(
        host.navigate("/integrations/example/Reserved").target(),
        Some("reserved")
    );
}

#[test]
fn duplicate_activation_and_failed_teardown_preserve_the_host_and_other_plugins() {
    let mut host = manager();
    publish_point(&mut host);
    let first = PluginId::new("first").unwrap();
    let second = PluginId::new("second").unwrap();
    let mut failing = Settings::new(&["Display"]);
    failing.fail_deactivate = true;
    host.activate(first.clone(), failing).unwrap();
    host.activate(second.clone(), Settings::new(&["Display"]))
        .unwrap();
    assert_eq!(
        host.activate(first.clone(), Settings::new(&["Replacement"])),
        Err(PluginApiError::AlreadyActive)
    );
    assert_eq!(
        host.deactivate(&first),
        Err(PluginApiError::Rejected("teardown failed".into()))
    );
    assert_eq!(host.state(&first), Some(PluginLifecycleState::Deactivated));
    assert_eq!(host.state(&second), Some(PluginLifecycleState::Active));
    assert!(!host.navigate("/integrations/first/Display").is_matched());
    assert!(host.navigate("/integrations/second/Display").is_matched());
    assert_eq!(host.contributions().views().len(), 1);
}

#[test]
fn a_panicking_plugin_does_not_publish_staged_pages_or_break_host_navigation() {
    struct Panics {
        during_initialize: bool,
    }

    impl PluginApi for Panics {
        fn initialize(
            &mut self,
            registration: &mut PluginRegistration,
        ) -> Result<(), PluginApiError> {
            registration.contribute_view(
                point(),
                ViewContribution::new(ContributionId::new("Page").unwrap(), "Page"),
                || panic!("render panic"),
            );
            assert!(!self.during_initialize, "initialize panic after staging");
            Ok(())
        }
    }

    let mut host = manager();
    publish_point(&mut host);
    let id = PluginId::new("panics").unwrap();
    assert_eq!(
        host.activate(
            id.clone(),
            Panics {
                during_initialize: true
            }
        ),
        Err(PluginApiError::Panicked)
    );
    assert!(host.contributions().views().is_empty());

    host.activate(
        id.clone(),
        Panics {
            during_initialize: false,
        },
    )
    .unwrap();
    assert!(host.navigate("/integrations/panics/Page").is_matched());
    assert_eq!(host.current_view(), Err(PluginApiError::Panicked));
    host.deactivate(&id).unwrap();
    assert!(host.navigate("/settings").is_matched());
}

struct VersionedSettings {
    text: &'static str,
    stopped: std::rc::Rc<std::cell::Cell<bool>>,
    fail_render: bool,
    fail_stop: bool,
}

impl PluginApi for VersionedSettings {
    fn initialize(&mut self, registration: &mut PluginRegistration) -> Result<(), PluginApiError> {
        let text = self.text;
        let fail_render = self.fail_render;
        registration.contribute_view(
            point(),
            ViewContribution::new(ContributionId::new("Page").unwrap(), text),
            move || {
                assert!(!fail_render, "first frame rejected");
                View::single(ViewNode::text(text))
            },
        );
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        self.stopped.set(true);
        if self.fail_stop {
            return Err(PluginApiError::Rejected("stop failed".into()));
        }
        Ok(())
    }
}

#[test]
fn confirms_handover_after_rendering_and_preserves_navigation() {
    let mut host = manager();
    publish_point(&mut host);
    let id = PluginId::new("example").unwrap();
    let old_stopped = std::rc::Rc::default();
    let new_stopped = std::rc::Rc::default();
    host.activate(
        id.clone(),
        VersionedSettings {
            text: "old",
            stopped: std::rc::Rc::clone(&old_stopped),
            fail_render: false,
            fail_stop: true,
        },
    )
    .unwrap();
    assert!(host.navigate("/settings").is_matched());
    assert!(
        host.navigate("/integrations/example/Page?mode=trial")
            .is_matched()
    );
    assert!(matches!(
        host.stage_handover(&id, Settings::new(&["Other"])),
        Err(PluginApiError::Rejected(_))
    ));
    assert!(!old_stopped.get());
    let handover = host
        .stage_handover(
            &id,
            VersionedSettings {
                text: "new",
                stopped: std::rc::Rc::clone(&new_stopped),
                fail_render: false,
                fail_stop: false,
            },
        )
        .unwrap();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let view = handover.trial().current_view().unwrap().unwrap();
    runtime.render_now(&view, &[]);
    assert!(
        runtime
            .tree()
            .iter()
            .any(|(_, node)| node.kind().text() == Some("new"))
    );
    assert!(!old_stopped.get());
    assert_eq!(
        handover.commit(),
        Err(PluginApiError::Rejected("stop failed".into()))
    );
    assert!(old_stopped.get());
    assert!(!new_stopped.get());
    assert_eq!(host.back().unwrap().target(), Some("settings"));
    assert_eq!(
        host.forward().unwrap().target(),
        Some("app.views/example/Page")
    );
    assert_eq!(host.contributions().views()[0].definition().title(), "new");
}

#[test]
fn dropping_failed_handover_restores_active_plugin_and_stops_candidate() {
    let mut host = manager();
    publish_point(&mut host);
    let id = PluginId::new("example").unwrap();
    let old_stopped = std::rc::Rc::default();
    let new_stopped = std::rc::Rc::default();
    host.activate(
        id.clone(),
        VersionedSettings {
            text: "old",
            stopped: std::rc::Rc::clone(&old_stopped),
            fail_render: false,
            fail_stop: false,
        },
    )
    .unwrap();
    assert!(host.navigate("/integrations/example/Page").is_matched());
    let handover = host
        .stage_handover(
            &id,
            VersionedSettings {
                text: "new",
                stopped: std::rc::Rc::clone(&new_stopped),
                fail_render: true,
                fail_stop: true,
            },
        )
        .unwrap();
    assert_eq!(
        handover.trial().current_view(),
        Err(PluginApiError::Panicked)
    );
    drop(handover);
    assert!(new_stopped.get());
    assert!(!old_stopped.get());
    assert!(host.current_view().unwrap().is_some());
    assert_eq!(host.contributions().views()[0].definition().title(), "old");
}

#[test]
fn applications_choose_multiple_points_routes_and_mount_policies() {
    struct Views(Vec<ExtensionPoint>);

    impl PluginApi for Views {
        fn initialize(
            &mut self,
            registrations: &mut PluginRegistration,
        ) -> Result<(), PluginApiError> {
            for point in &self.0 {
                registrations.contribute_view(
                    point.clone(),
                    ViewContribution::new(ContributionId::new("shared").unwrap(), "Example"),
                    || View::single(ViewNode::text("Example")),
                );
            }
            Ok(())
        }
    }

    let routed = ExtensionPoint::new("workspace.pages").unwrap();
    let inline = ExtensionPoint::new("dashboard.cards").unwrap();
    let restricted = ExtensionPoint::new("workspace.restricted").unwrap();
    let conflict = ExtensionPoint::new("workspace.conflict").unwrap();
    let mut host = manager();
    assert!(host.publish(inline.clone()));
    assert!(host.publish_with(routed.clone(), |_, _| Ok(Some(Route::new(
        "/tools/example"
    )))));
    assert!(!host.publish(routed.clone()));
    assert!(
        host.publish_with(conflict.clone(), |_, _| Ok(Some(Route::new(
            "/tools/example"
        ))))
    );
    assert!(
        host.publish_with(restricted.clone(), |_, _| Err(PluginApiError::Rejected(
            "not allowed here".into()
        )))
    );
    let id = PluginId::new("example").unwrap();

    assert_eq!(
        host.activate(id.clone(), Views(vec![inline.clone(), restricted])),
        Err(PluginApiError::Rejected("not allowed here".into()))
    );
    assert!(host.contributions().views().is_empty());
    assert_eq!(
        host.activate(id.clone(), Views(vec![routed.clone(), conflict])),
        Err(PluginApiError::RouteConflict("/tools/example".into()))
    );
    assert!(host.contributions().views().is_empty());

    host.activate(id.clone(), Views(vec![routed.clone(), inline.clone()]))
        .unwrap();
    let card = host.contributions().views_at(&inline).next().unwrap();
    assert!(card.route().is_none());
    assert!(card.render().is_ok());
    assert_eq!(
        host.navigate("/tools/example").target(),
        Some("workspace.pages/example/shared")
    );
    assert!(host.current_view().unwrap().is_some());

    host.deactivate(&id).unwrap();
    assert_eq!(host.contributions().views_at(&inline).count(), 0);
    assert!(host.current_view().unwrap().is_none());
}
