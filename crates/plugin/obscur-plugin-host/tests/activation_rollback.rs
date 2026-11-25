//! Activation rollback and host inspection across lifecycle transitions.

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use obscur_plugin_api::{
    plugin::{PluginApi, PluginRegistration},
    status::PluginApiError,
};
use obscur_plugin_host::{lifecycle::CrashRecovery, manager::PluginManager};
use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::{ContributionId, PluginId},
    lifecycle::{PluginLifecycleState, ShutdownConcern},
    manifest::Version,
};
use obscur_routing::router::RouteRegistry;
use obscur_view::view::{View, ViewNode};

struct Plugin {
    point: ExtensionPoint,
    initialize_fails: bool,
    deactivations: Rc<Cell<u32>>,
}

struct CrashingPlugin;

impl PluginApi for CrashingPlugin {
    fn initialize(&mut self, _: &mut PluginRegistration) -> Result<(), PluginApiError> {
        Err(PluginApiError::ExecutorCrashed("guest fault".into()))
    }
}

#[test]
fn configured_recovery_defers_and_blocks_activation_before_running_the_plugin() {
    let id = PluginId::new("example.plugin").unwrap();
    let mut host = PluginManager::new(RouteRegistry::new());
    let delay = Duration::from_secs(600);
    host.set_crash_recovery(id.clone(), CrashRecovery::new(delay, delay, 2));
    assert!(matches!(
        host.activate(id.clone(), CrashingPlugin),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
    assert!(matches!(
        host.activate(id.clone(), CrashingPlugin),
        Err(PluginApiError::RestartDeferred(remaining)) if !remaining.is_zero() && remaining <= delay
    ));

    host.set_crash_recovery(
        id.clone(),
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 1),
    );
    assert!(matches!(
        host.activate(id.clone(), CrashingPlugin),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
    assert_eq!(
        host.activate(id.clone(), CrashingPlugin),
        Err(PluginApiError::RestartBlocked)
    );
    host.crash_recovery_mut(&id).unwrap().reset();
    assert!(matches!(
        host.activate(id, CrashingPlugin),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
}

impl PluginApi for Plugin {
    fn shutdown_requested(
        &mut self,
        _: Instant,
    ) -> Result<Option<ShutdownConcern>, PluginApiError> {
        Ok(Some(ShutdownConcern {
            message: "saving".into(),
            requested_time: Duration::MAX,
        }))
    }

    fn initialize(&mut self, registration: &mut PluginRegistration) -> Result<(), PluginApiError> {
        registration.contribute_view(
            self.point.clone(),
            ViewContribution::new(ContributionId::new("demo").unwrap(), "Demo"),
            || View::single(ViewNode::text("Demo")),
        );
        if self.initialize_fails {
            return Err(PluginApiError::Rejected("initialize failed".into()));
        }
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        self.deactivations.set(self.deactivations.get() + 1);
        Err(PluginApiError::Rejected("cleanup failed".into()))
    }
}

#[test]
fn shutdown_caps_the_concern_and_withdraws_active_views_even_on_teardown_error() {
    let point = ExtensionPoint::new("app.tools").unwrap();
    let id = PluginId::new("example.plugin").unwrap();
    let mut host = PluginManager::new(RouteRegistry::new());
    host.publish(point.clone());
    host.activate(
        id.clone(),
        Plugin {
            point,
            initialize_fails: false,
            deactivations: Rc::default(),
        },
    )
    .unwrap();

    assert!(host.shutdown_requested(Instant::now()).is_empty());
    let budget = Duration::from_secs(10);
    let responses = host.shutdown_requested(Instant::now() + budget);
    assert!(matches!(
        responses.as_slice(),
        [(owner, Ok(Some(concern)))] if owner == &id && concern.requested_time <= budget
    ));
    assert_eq!(
        host.deactivate_all(),
        vec![(
            id.clone(),
            PluginApiError::Rejected("cleanup failed".into())
        )]
    );
    assert_eq!(host.state(&id), Some(PluginLifecycleState::Deactivated));
    assert!(host.contributions().views().is_empty());
}

#[test]
fn failed_activation_deactivates_the_candidate_and_keeps_the_original_error() {
    let point = ExtensionPoint::new("app.tools").unwrap();
    let id = PluginId::new("example.plugin").unwrap();
    let deactivations = Rc::new(Cell::new(0));
    let plugin = |initialize_fails| Plugin {
        point: point.clone(),
        initialize_fails,
        deactivations: Rc::clone(&deactivations),
    };
    let mut host = PluginManager::new(RouteRegistry::new());

    assert!(matches!(
        host.activate(id.clone(), plugin(true)),
        Err(PluginApiError::Rejected(reason)) if reason.as_ref() == "initialize failed"
    ));
    assert_eq!(deactivations.get(), 1);
    assert!(matches!(
        host.activate(id.clone(), plugin(false)),
        Err(PluginApiError::ExtensionUnavailable)
    ));
    assert_eq!(deactivations.get(), 2);
    assert!(host.state(&id).is_none());
    assert!(host.contributions().views().is_empty());

    assert!(host.publish(point.clone()));
    assert!(host.activate(id.clone(), plugin(false)).is_ok());
    assert_eq!(host.state(&id), Some(PluginLifecycleState::Active));
    assert_eq!(host.contributions().views_at(&point).count(), 1);
    assert_eq!(deactivations.get(), 2);
}

#[test]
fn inspection_tracks_the_version_across_handover_and_deactivation() {
    let point = ExtensionPoint::new("app.tools").unwrap();
    let id = PluginId::new("example.plugin").unwrap();
    let make_plugin = || Plugin {
        point: point.clone(),
        initialize_fails: false,
        deactivations: Rc::default(),
    };
    let mut host = PluginManager::new(RouteRegistry::new());
    host.publish(point.clone());
    let first = Version::new(1, 0, 0);
    let second = Version::new(2, 0, 0);
    host.activate_with_version(id.clone(), first.clone(), make_plugin())
        .unwrap();
    host.set_crash_recovery(
        id.clone(),
        CrashRecovery::new(Duration::ZERO, Duration::ZERO, 1),
    );
    assert!(matches!(
        host.stage_handover_with_version(&id, second.clone(), CrashingPlugin),
        Err(PluginApiError::ExecutorCrashed(_))
    ));
    assert_eq!(
        host.last_failure(&id).unwrap().version.as_ref(),
        Some(&second)
    );
    assert_eq!(
        host.plugin_states().next(),
        Some((&id, Some(&first), PluginLifecycleState::Active))
    );
    assert_eq!(
        host.crash_recovery_mut(&id)
            .unwrap()
            .retry_after(Instant::now()),
        Some(Duration::ZERO)
    );
    let handover = host
        .stage_handover_with_version(&id, second.clone(), make_plugin())
        .unwrap();
    assert_eq!(
        handover.trial().plugin_states().next(),
        Some((&id, Some(&second), PluginLifecycleState::Active))
    );
    assert!(handover.rollback().is_err());
    assert_eq!(
        host.plugin_states().next(),
        Some((&id, Some(&first), PluginLifecycleState::Active))
    );
    let handover = host
        .stage_handover_with_version(&id, second.clone(), make_plugin())
        .unwrap();
    assert!(handover.commit().is_err());
    assert!(host.deactivate(&id).is_err());
    assert_eq!(
        host.plugin_states().next(),
        Some((&id, Some(&second), PluginLifecycleState::Deactivated))
    );
}
