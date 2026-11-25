//! `PluginManager` (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use obscur_plugin_api::{
    plugin::{PluginApi, PluginRegistration},
    status::PluginApiError,
};
use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::{ContributionId, PluginId},
    lifecycle::{PluginLifecycleState, ShutdownConcern},
    manifest::{PluginManifest, Version},
};
use obscur_routing::route::Route;
use obscur_routing::router::{NavigationResult, RouteRegistry, Router};
use obscur_view::view::View;

use super::{PluginFailure, PluginRecord};
use crate::activation::ActivationTransaction;
use crate::contribution::{ContributionRegistry, ExtensionPointRegistry, MountedView};
use crate::lifecycle::{CrashRecovery, PluginSuspension};
use crate::update::StagedHandover;

/// A host for built-in trusted plugin activation, contribution routing, and teardown.
pub struct PluginManager {
    plugins: BTreeMap<PluginId, PluginRecord>,
    recovery: BTreeMap<PluginId, CrashRecovery>,
    failures: BTreeMap<PluginId, PluginFailure>,
    points: ExtensionPointRegistry,
    contributions: ContributionRegistry,
    host_routes: RouteRegistry,
    router: Router,
}

impl PluginManager {
    /// Creates a manager with the application’s existing route registry.
    #[must_use]
    pub fn new(host_routes: RouteRegistry) -> Self {
        let router = Router::new(host_routes.clone());
        Self {
            plugins: BTreeMap::new(),
            recovery: BTreeMap::new(),
            failures: BTreeMap::new(),
            points: ExtensionPointRegistry::default(),
            contributions: ContributionRegistry::default(),
            host_routes,
            router,
        }
    }

    /// Publishes a view integration point with no automatic route. Returns false if the point was already published.
    pub fn publish(&mut self, point: ExtensionPoint) -> bool {
        self.publish_with(point, |_, _| Ok(None))
    }

    /// Publishes a point with application-controlled validation and routing.
    /// Return `None` for views inserted directly by the application.
    pub fn publish_with(
        &mut self,
        point: ExtensionPoint,
        mount: impl Fn(&PluginId, &ViewContribution) -> Result<Option<Route>, PluginApiError> + 'static,
    ) -> bool {
        self.points.publish(point, mount)
    }

    /// Publishes an application integration point with a versioned contract and mount policy. A contribution declaring an incompatible contract is skipped during activation.
    pub fn publish_versioned(
        &mut self,
        point: ExtensionPoint,
        version: Version,
        mount: impl Fn(&PluginId, &ViewContribution) -> Result<Option<Route>, PluginApiError> + 'static,
    ) -> bool {
        self.points.publish_versioned(point, Some(version), mount)
    }

    /// Stages and validates all registrations before publishing the plugin’s contributions.
    pub fn activate(
        &mut self,
        id: PluginId,
        plugin: impl PluginApi + 'static,
    ) -> Result<(), PluginApiError> {
        self.activate_with_version(id, None, plugin)
    }

    /// Activates a plugin and records its version for host inspection.
    pub fn activate_with_version(
        &mut self,
        id: PluginId,
        version: impl Into<Option<Version>>,
        plugin: impl PluginApi + 'static,
    ) -> Result<(), PluginApiError> {
        self.activate_from(id, version.into(), || Ok(plugin))
    }

    /// Loads and activates a plugin selected by manifest. The host-supplied callback verifies and loads the matching package. It is only called if activation is currently permitted.
    pub fn activate_manifest<P: PluginApi + 'static>(
        &mut self,
        manifest: &PluginManifest,
        load: impl FnOnce(&PluginManifest) -> Result<P, PluginApiError>,
    ) -> Result<(), PluginApiError> {
        self.activate_from(manifest.id.clone(), Some(manifest.version.clone()), || {
            load(manifest)
        })
    }

    /// Opens a view selected by the application. Loads the selected plugin on first
    /// use, resumes it when suspended, and reuses an already active instance.
    /// Returns no view if the plugin does not register that contribution at the
    /// requested point. The caller owns the loading and error UI.
    pub fn open_manifest_view<P: PluginApi + 'static>(
        &mut self,
        manifest: &PluginManifest,
        point: &ExtensionPoint,
        view: &ContributionId,
        load: impl FnOnce(&PluginManifest) -> Result<P, PluginApiError>,
    ) -> Result<Option<View>, PluginApiError> {
        self.open_manifest_view_with(manifest, point, view, |manager| {
            manager.activate_manifest(manifest, load)
        })
    }

    #[cfg(feature = "plugin-wasm")]
    /// Opens a WASM view selected by the application. On first use, loads a
    /// validated guest that must match the selected manifest and attaches its
    /// managed fault handling. Reuses or resumes an already loaded instance
    /// of the selected version.
    pub fn open_wasm_manifest_view(
        &mut self,
        manifest: &PluginManifest,
        point: &ExtensionPoint,
        view: &ContributionId,
        load: impl FnOnce(
            &PluginManifest,
        ) -> Result<
            obscur_plugin_executor_wasm::executor::ManagedWasmPlugin,
            PluginApiError,
        >,
    ) -> Result<Option<View>, PluginApiError> {
        self.open_manifest_view_with(manifest, point, view, |manager| {
            let mut handle = None;
            manager.activate_manifest(manifest, |selected| {
                let guest = load(selected)?;
                if guest.manifest() != selected {
                    return Err(PluginApiError::Rejected(
                        "WASM package does not match the selected manifest".into(),
                    ));
                }
                handle = Some(guest.handle());
                Ok(guest)
            })?;
            if let Some(handle) = handle {
                manager.attach_wasm(handle);
            }
            Ok(())
        })
    }

    fn open_manifest_view_with(
        &mut self,
        manifest: &PluginManifest,
        point: &ExtensionPoint,
        view: &ContributionId,
        activate: impl FnOnce(&mut Self) -> Result<(), PluginApiError>,
    ) -> Result<Option<View>, PluginApiError> {
        let id = &manifest.id;
        match self
            .plugins
            .get(id)
            .filter(|record| record.instance.is_some())
        {
            Some(record) => {
                if record.version.as_ref() != Some(&manifest.version) {
                    return Err(PluginApiError::Rejected(
                        "a different plugin version is already loaded".into(),
                    ));
                }
                if record.state() == PluginLifecycleState::Suspended {
                    self.resume(id)?;
                }
            }
            None => activate(self)?,
        }
        self.contributions
            .views_at(point)
            .find(|mounted| mounted.owner() == id && mounted.definition().id() == view)
            .map(MountedView::render)
            .transpose()
    }

    fn activate_from<P: PluginApi + 'static>(
        &mut self,
        id: PluginId,
        version: Option<Version>,
        load: impl FnOnce() -> Result<P, PluginApiError>,
    ) -> Result<(), PluginApiError> {
        if self
            .plugins
            .get(&id)
            .is_some_and(|record| record.instance.is_some())
        {
            return Err(PluginApiError::AlreadyActive);
        }
        if let Some(recovery) = self.recovery.get(&id) {
            let delay = recovery
                .retry_after(Instant::now())
                .ok_or(PluginApiError::RestartBlocked)?;
            if !delay.is_zero() {
                return Err(PluginApiError::RestartDeferred(delay));
            }
        }
        let loaded = catch_unwind(AssertUnwindSafe(load))
            .map_err(|_| PluginApiError::Panicked)
            .and_then(|result| result);
        let plugin = self.track_failure(&id, version.clone(), loaded)?;
        let prepared = self.prepare(&id, version.clone(), plugin, self.router.registry());
        let (record, pages) = self.track_failure(&id, version, prepared)?;
        self.contributions.publish(pages);
        self.rebuild_routes();
        self.plugins.insert(id, record);
        Ok(())
    }

    /// Activates a managed WASM plugin under its verified manifest ID.
    #[cfg(feature = "plugin-wasm")]
    pub fn activate_wasm(
        &mut self,
        plugin: obscur_plugin_executor_wasm::executor::ManagedWasmPlugin,
    ) -> Result<(), PluginApiError> {
        let handle = plugin.handle();
        self.activate_with_version(plugin.id().clone(), plugin.version().clone(), plugin)?;
        self.attach_wasm(handle);
        Ok(())
    }

    #[cfg(feature = "plugin-wasm")]
    /// Activates a fresh WASM instance with the latest committed snapshot as its
    /// initialization request. The caller supplies the host-assigned installation
    /// namespace and the state schema range accepted by the guest. With no committed
    /// snapshot, the instance uses its configured initialization request.
    pub fn activate_wasm_from_state(
        &mut self,
        plugin: obscur_plugin_executor_wasm::executor::ManagedWasmPlugin,
        store: &impl crate::storage::StateStore,
        namespace: &crate::storage::StorageNamespace,
        accepted: &obscur_plugin_model::manifest::VersionReq,
    ) -> Result<(), PluginApiError> {
        if namespace.plugin() != plugin.id() {
            return Err(PluginApiError::Rejected(
                "state belongs to another plugin".into(),
            ));
        }
        let snapshot =
            crate::storage::load_compatible_state(store, namespace, accepted).map_err(|error| {
                PluginApiError::Rejected(
                    format!("could not load committed plugin state: {error}").into(),
                )
            })?;
        let plugin = if let Some(snapshot) = snapshot {
            plugin.with_initial_request(snapshot.bytes().to_vec())
        } else {
            plugin
        };
        self.activate_wasm(plugin)
    }

    /// Stages a WASM replacement using its verified manifest ID.
    #[cfg(feature = "plugin-wasm")]
    pub fn stage_wasm_handover(
        &mut self,
        plugin: obscur_plugin_executor_wasm::executor::ManagedWasmPlugin,
    ) -> Result<StagedHandover<'_>, PluginApiError> {
        let handle = plugin.handle();
        let mut staged = self.stage_handover_with_version(
            &plugin.id().clone(),
            plugin.version().clone(),
            plugin,
        )?;
        staged.attach_wasm(handle);
        Ok(staged)
    }

    #[cfg(feature = "plugin-wasm")]
    pub(crate) fn attach_wasm(
        &mut self,
        handle: obscur_plugin_executor_wasm::executor::ManagedWasmHandle,
    ) {
        if let Some(record) = self.plugins.get_mut(handle.id()) {
            record.wasm = Some(handle);
        }
    }

    #[cfg(feature = "plugin-wasm")]
    pub(crate) fn wasm_faulted(&self, id: &PluginId) -> bool {
        self.plugins
            .get(id)
            .and_then(|record| record.wasm.as_ref())
            .is_some_and(|handle| handle.is_faulted())
    }

    #[cfg(feature = "plugin-wasm")]
    /// Withdraws contributions and routes of faulted managed WASM instances and applies
    /// their restart policies. Returns the affected IDs and their original faults.
    /// Call from the host event loop after processing guest messages.
    pub fn withdraw_faulted_wasm(&mut self) -> Vec<(PluginId, PluginApiError)> {
        let faulted: Vec<_> = self
            .plugins
            .iter()
            .filter_map(|(id, record)| {
                (record.instance.is_some()
                    && record
                        .wasm
                        .as_ref()
                        .is_some_and(|handle| handle.is_faulted()))
                .then(|| id.clone())
            })
            .collect();
        faulted
            .into_iter()
            .filter_map(|id| self.deactivate(&id).err().map(|error| (id, error)))
            .collect()
    }

    /// Withdraws UI and routes before invoking plugin teardown. A teardown error leaves the plugin inactive.
    pub fn deactivate(&mut self, id: &PluginId) -> Result<(), PluginApiError> {
        self.deactivate_with_deadline(id, None)
    }

    /// Withdraws contributions and routes while retaining the active instance for resumption. Managed executors pause message delivery.
    pub fn suspend(&mut self, id: &PluginId) -> Result<(), PluginApiError> {
        let record = self
            .plugins
            .get_mut(id)
            .filter(|record| record.suspension.is_none())
            .ok_or(PluginApiError::NotActive)?;
        let plugin = record.instance.as_mut().ok_or(PluginApiError::NotActive)?;
        catch_unwind(AssertUnwindSafe(|| plugin.suspend()))
            .map_err(|_| PluginApiError::Panicked)?;
        record.suspension = Some(PluginSuspension::new(self.contributions.take_views(id)));
        self.rebuild_routes();
        Ok(())
    }

    /// Restores a suspended instance and its registered views without initialization. If any retained route is now occupied, the plugin remains suspended.
    pub fn resume(&mut self, id: &PluginId) -> Result<(), PluginApiError> {
        let record = self.plugins.get_mut(id).ok_or(PluginApiError::NotActive)?;
        let pages = record
            .suspension
            .as_ref()
            .ok_or(PluginApiError::NotActive)?;
        for page in &pages.views {
            if let Some(route) = page.route()
                && self.router.registry().resolve(route).is_some()
            {
                return Err(PluginApiError::RouteConflict(route.path().into()));
            }
        }
        let plugin = record.instance.as_mut().ok_or(PluginApiError::NotActive)?;
        catch_unwind(AssertUnwindSafe(|| plugin.resume())).map_err(|_| PluginApiError::Panicked)?;
        if let Some(suspension) = record.suspension.take() {
            self.contributions.publish(suspension.views);
        }
        self.rebuild_routes();
        Ok(())
    }

    fn deactivate_with_deadline(
        &mut self,
        id: &PluginId,
        deadline: Option<Instant>,
    ) -> Result<(), PluginApiError> {
        let Some((plugin, version)) = self.plugins.get_mut(id).and_then(|record| {
            let instance = record.instance.take()?;
            record.suspension = None;
            Some((instance, record.version.clone()))
        }) else {
            return Err(PluginApiError::NotActive);
        };
        self.contributions.withdraw(id);
        self.rebuild_routes();
        let result = PluginRecord::deactivate_instance(plugin, deadline);
        self.track_failure(id, version, result)
    }

    /// Installs a restart policy for the plugin ID. Activation and deactivation
    /// record panics, executor crashes, and execution quota failures. Explicit
    /// handover of an active plugin is a separate transaction.
    pub fn set_crash_recovery(&mut self, id: PluginId, recovery: CrashRecovery) {
        self.recovery.insert(id, recovery);
    }

    /// Asks active plugins for advisory shutdown concerns before the host deadline.
    /// Requested times are capped to the remaining budget. Callbacks are synchronous and trusted in-process plugins must return promptly.
    pub fn shutdown_requested(
        &mut self,
        deadline: Instant,
    ) -> Vec<(PluginId, Result<Option<ShutdownConcern>, PluginApiError>)> {
        let mut responses = Vec::new();
        for (id, record) in &mut self.plugins {
            if Instant::now() >= deadline {
                break;
            }
            if let Some(plugin) = record.instance.as_mut() {
                let response =
                    catch_unwind(AssertUnwindSafe(|| plugin.shutdown_requested(deadline)))
                        .map_err(|_| PluginApiError::Panicked)
                        .and_then(|response| response)
                        .map(|concern| {
                            concern.map(|mut concern| {
                                concern.requested_time = concern
                                    .requested_time
                                    .min(deadline.saturating_duration_since(Instant::now()));
                                concern
                            })
                        });
                responses.push((id.clone(), response));
            }
        }
        responses
    }

    /// Withdraws and deactivates all active plugins. Returns teardown errors by plugin ID after attempting every active plugin.
    pub fn deactivate_all(&mut self) -> Vec<(PluginId, PluginApiError)> {
        self.deactivate_all_with_deadline(None)
    }

    /// Unloads plugins suspended for at least the supplied idle interval. The host supplies a monotonic observation time. Returns the ID and result for each attempted unload.
    pub fn unload_idle(
        &mut self,
        now: Instant,
        idle_for: Duration,
    ) -> Vec<(PluginId, Result<(), PluginApiError>)> {
        let expired: Vec<_> = self
            .plugins
            .iter()
            .filter_map(|(id, record)| {
                record
                    .suspension
                    .as_ref()
                    .filter(|suspension| suspension.expired(now, idle_for))
                    .map(|_| id.clone())
            })
            .collect();
        expired
            .into_iter()
            .map(|id| {
                let result = self.deactivate(&id);
                (id, result)
            })
            .collect()
    }

    /// Withdraws all active contributions and gives each executor the same shutdown
    /// deadline. Expired WASM hooks are skipped and guest instances are dropped.
    /// Trusted in-process callbacks and destructors must cooperate with shutdown.
    pub fn deactivate_all_until(&mut self, deadline: Instant) -> Vec<(PluginId, PluginApiError)> {
        self.deactivate_all_with_deadline(Some(deadline))
    }

    fn deactivate_all_with_deadline(
        &mut self,
        deadline: Option<Instant>,
    ) -> Vec<(PluginId, PluginApiError)> {
        let active: Vec<_> = self
            .plugins
            .iter()
            .filter_map(|(id, record)| record.instance.as_ref().map(|_| id.clone()))
            .collect();
        active
            .into_iter()
            .filter_map(|id| {
                self.deactivate_with_deadline(&id, deadline)
                    .err()
                    .map(|error| (id, error))
            })
            .collect()
    }

    /// Accesses the installed restart policy so the host can report runtime
    /// failures observed by its executor driver or reset the policy.
    pub fn crash_recovery_mut(&mut self, id: &PluginId) -> Option<&mut CrashRecovery> {
        self.recovery.get_mut(id)
    }

    fn track_failure<T>(
        &mut self,
        id: &PluginId,
        version: Option<Version>,
        result: Result<T, PluginApiError>,
    ) -> Result<T, PluginApiError> {
        if let Err(error) = &result
            && self.record_failure(id, version, error)
            && let Some(recovery) = self.recovery.get_mut(id)
        {
            recovery.record_failure(Instant::now());
        }
        result
    }

    fn record_failure(
        &mut self,
        id: &PluginId,
        version: Option<Version>,
        error: &PluginApiError,
    ) -> bool {
        if !matches!(
            error,
            PluginApiError::Panicked
                | PluginApiError::ExecutionTimedOut
                | PluginApiError::ExecutorCrashed(_)
                | PluginApiError::ExecutionQuotaExceeded(_)
        ) {
            return false;
        }
        self.failures.insert(
            id.clone(),
            PluginFailure {
                version,
                error: error.clone(),
            },
        );
        true
    }

    /// Returns the last recorded panic, executor crash, or quota failure for this ID.
    /// The diagnostic remains available after deactivation or successful restart.
    #[must_use]
    pub fn last_failure(&self, id: &PluginId) -> Option<&PluginFailure> {
        self.failures.get(id)
    }

    /// Mounts a replacement provisionally while retaining the previous instance.
    ///
    /// Render and confirm the first frame through the returned handover. The
    /// current plugin route must remain present if it belongs to this plugin.
    pub fn stage_handover(
        &mut self,
        id: &PluginId,
        candidate: impl PluginApi + 'static,
    ) -> Result<StagedHandover<'_>, PluginApiError> {
        self.stage_handover_with_version(id, None, candidate)
    }

    /// Stages a replacement and its version for host inspection. Rolling back restores the previous version.
    pub fn stage_handover_with_version(
        &mut self,
        id: &PluginId,
        version: impl Into<Option<Version>>,
        candidate: impl PluginApi + 'static,
    ) -> Result<StagedHandover<'_>, PluginApiError> {
        if self.state(id) != Some(PluginLifecycleState::Active) {
            return Err(PluginApiError::NotActive);
        }
        let routes = self.routes_excluding(Some(id));
        let version = version.into();
        let (candidate, pages) = self
            .prepare(id, version.clone(), candidate, &routes)
            .inspect_err(|error| {
                self.record_failure(id, version, error);
            })?;
        if let Some(entry) = self.router.current() {
            let path = entry.route().path();
            let owns_current = self.contributions.views().iter().any(|page| {
                page.owner() == id
                    && page
                        .pattern()
                        .is_some_and(|pattern| pattern.match_path(path).is_some())
            });
            if owns_current
                && !pages.iter().any(|page| {
                    page.pattern()
                        .is_some_and(|pattern| pattern.match_path(path).is_some())
                })
            {
                drop(pages);
                let error = PluginApiError::Rejected(
                    format!("replacement removes current plugin route {path}").into(),
                );
                let _ = self.deactivate_handover_record(id, candidate);
                return Err(error);
            }
        }

        let record = self.plugins.get_mut(id).ok_or(PluginApiError::NotActive)?;
        let previous = std::mem::replace(record, candidate);
        let previous_pages = self.replace_views(id, pages);
        Ok(StagedHandover::new(
            self,
            id.clone(),
            previous,
            previous_pages,
        ))
    }

    pub(crate) fn restore_handover(
        &mut self,
        id: PluginId,
        previous: PluginRecord,
        pages: Vec<MountedView>,
    ) -> Result<(), PluginApiError> {
        let candidate = self.plugins.insert(id.clone(), previous);
        self.replace_views(&id, pages);
        candidate.map_or(Ok(()), |candidate| {
            self.deactivate_handover_record(&id, candidate)
        })
    }

    pub(crate) fn deactivate_handover_record(
        &mut self,
        id: &PluginId,
        record: PluginRecord,
    ) -> Result<(), PluginApiError> {
        let version = record.version.clone();
        record.deactivate().inspect_err(|error| {
            self.record_failure(id, version, error);
        })
    }

    /// Returns the last committed lifecycle state, if any.
    #[must_use]
    pub fn state(&self, id: &PluginId) -> Option<PluginLifecycleState> {
        self.plugins.get(id).map(PluginRecord::state)
    }

    /// Lists plugins by ID with their recorded versions and lifecycle states without invoking plugin code.
    /// An unversioned registration reports no version. A deactivated plugin retains the last activated version.
    pub fn plugin_states(
        &self,
    ) -> impl Iterator<Item = (&PluginId, Option<&Version>, PluginLifecycleState)> {
        self.plugins
            .iter()
            .map(|(id, record)| (id, record.version.as_ref(), record.state()))
    }

    /// Returns the host’s active contribution catalog.
    #[must_use]
    pub const fn contributions(&self) -> &ContributionRegistry {
        &self.contributions
    }

    /// Provides read access to the normal navigation history and effective route registry.
    #[must_use]
    pub const fn router(&self) -> &Router {
        &self.router
    }

    /// Navigates through the shared host and active plugin route registry.
    pub fn navigate(&mut self, location: &str) -> NavigationResult {
        self.router.navigate(location)
    }

    /// Moves backward through the normal navigation history.
    pub fn back(&mut self) -> Option<NavigationResult> {
        self.router.back()
    }

    /// Moves forward through the normal navigation history.
    pub fn forward(&mut self) -> Option<NavigationResult> {
        self.router.forward()
    }

    /// Builds the active route’s plugin view, if the route belongs to a plugin.
    pub fn current_view(&self) -> Result<Option<View>, PluginApiError> {
        let Some(entry) = self.router.current() else {
            return Ok(None);
        };
        let page = self.contributions.views().iter().find(|page| {
            page.pattern()
                .is_some_and(|pattern| pattern.match_path(entry.route().path()).is_some())
        });
        page.map(MountedView::render).transpose()
    }

    fn prepare(
        &self,
        id: &PluginId,
        version: Option<Version>,
        mut plugin: impl PluginApi + 'static,
        routes: &RouteRegistry,
    ) -> Result<(PluginRecord, Vec<MountedView>), PluginApiError> {
        let mut staged = PluginRegistration::default();
        let pages = (|| {
            catch_unwind(AssertUnwindSafe(|| plugin.initialize(&mut staged)))
                .map_err(|_| PluginApiError::Panicked)??;
            ActivationTransaction::validate(id, staged, &self.points, routes)
        })();
        let plugin = Box::new(plugin);
        match pages {
            Ok(pages) => Ok((
                PluginRecord {
                    instance: Some(plugin),
                    version,
                    suspension: None,
                    #[cfg(feature = "plugin-wasm")]
                    wasm: None,
                },
                pages,
            )),
            Err(error) => {
                let _ = PluginRecord::deactivate_instance(plugin, None);
                Err(error)
            }
        }
    }

    fn replace_views(&mut self, id: &PluginId, pages: Vec<MountedView>) -> Vec<MountedView> {
        let previous = self.contributions.take_views(id);
        self.contributions.publish(pages);
        self.rebuild_routes();
        previous
    }

    fn rebuild_routes(&mut self) {
        self.router.replace_registry(self.routes_excluding(None));
    }

    fn routes_excluding(&self, owner: Option<&PluginId>) -> RouteRegistry {
        let mut routes = self.host_routes.clone();
        for page in self.contributions.views() {
            if Some(page.owner()) != owner
                && let Some(pattern) = page.pattern()
            {
                routes.register(pattern.clone(), page.global_id());
            }
        }
        routes
    }
}
