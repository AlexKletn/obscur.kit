use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use obscur_plugin_api::{
    plugin::{PluginApi, PluginRegistration},
    status::PluginApiError,
};
use obscur_plugin_model::{identity::PluginId, lifecycle::ShutdownConcern, manifest::Version};
use obscur_plugin_package::package::PluginPackage;

use super::instance_lifetime::InstanceLifetime;
use super::{ManagedWasmHandle, WasmLimits, WasmMessageLimits, WasmPluginInstance};
use crate::error::WasmExecutorError;

type RegistrationDecoder =
    Box<dyn FnOnce(&[u8], &mut PluginRegistration) -> Result<(), PluginApiError>>;

/// Adapter from the guest lifecycle to the host's transactional plugin API.
///
/// Trusted host code validates the initialization message and stages its
/// contributions using the same registration API as built-in plugins.
pub struct ManagedWasmPlugin {
    package: PluginPackage,
    limits: WasmLimits,
    message_limits: WasmMessageLimits,
    initial: Vec<u8>,
    max_host_call_bytes: Option<usize>,
    decode: Option<RegistrationDecoder>,
    instance: Rc<RefCell<Option<WasmPluginInstance>>>,
    lifetime: Rc<InstanceLifetime>,
}

impl ManagedWasmPlugin {
    /// Prepares a validated package and its host-owned decoder for activation.
    pub fn new(
        package: PluginPackage,
        limits: WasmLimits,
        message_limits: WasmMessageLimits,
        initial: Vec<u8>,
        decode: impl FnOnce(&[u8], &mut PluginRegistration) -> Result<(), PluginApiError> + 'static,
    ) -> Self {
        Self {
            package,
            limits,
            message_limits,
            initial,
            max_host_call_bytes: None,
            decode: Some(Box::new(decode)),
            instance: Rc::default(),
            lifetime: Rc::default(),
        }
    }

    /// Enables the optional guest host-call import and sets its packet budget.
    #[must_use]
    pub fn with_host_calls(mut self, max_call_bytes: usize) -> Self {
        self.max_host_call_bytes = Some(max_call_bytes);
        self
    }

    /// Replaces the guest initialization request before activation.
    #[must_use]
    pub fn with_initial_request(mut self, request: Vec<u8>) -> Self {
        self.initial = request;
        self
    }

    /// Returns a non-owning handle for host callbacks and other managed calls.
    #[must_use]
    pub fn handle(&self) -> ManagedWasmHandle {
        ManagedWasmHandle::new(
            self.id().clone(),
            Rc::downgrade(&self.instance),
            Rc::clone(&self.lifetime),
        )
    }

    /// Returns the plugin identifier from the validated package.
    #[must_use]
    pub fn id(&self) -> &PluginId {
        &self.package.manifest().id
    }

    /// Returns the version from the validated package.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.package.manifest().version
    }

    /// Returns the manifest from the validated package.
    #[must_use]
    pub fn manifest(&self) -> &obscur_plugin_model::manifest::PluginManifest {
        self.package.manifest()
    }
}

impl PluginApi for ManagedWasmPlugin {
    fn suspend(&mut self) {
        self.lifetime.set_suspended(true);
    }

    fn resume(&mut self) {
        self.lifetime.set_suspended(false);
    }

    fn shutdown_requested(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<ShutdownConcern>, PluginApiError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let budget_ms = u64::try_from(remaining.as_millis()).unwrap_or(u64::MAX);
        if budget_ms == 0 {
            return Ok(None);
        }
        let reply = self
            .handle()
            .invoke_guest(|instance| instance.shutdown_requested(budget_ms, deadline))
            .map_err(lifecycle_error)?;
        if reply.is_empty() {
            return Ok(None);
        }
        let invalid = || PluginApiError::Rejected("invalid WASM shutdown concern".into());
        let (time, message) = reply.split_first_chunk::<8>().ok_or_else(invalid)?;
        let message = std::str::from_utf8(message).map_err(|_| invalid())?;
        Ok(Some(ShutdownConcern {
            message: message.into(),
            requested_time: Duration::from_millis(u64::from_le_bytes(*time).min(budget_ms)),
        }))
    }

    fn initialize(&mut self, registration: &mut PluginRegistration) -> Result<(), PluginApiError> {
        let decode = self.decode.take().ok_or_else(|| {
            PluginApiError::Rejected("WASM plugin activation already attempted".into())
        })?;
        let activated = if let Some(max_call_bytes) = self.max_host_call_bytes {
            WasmPluginInstance::activate_with_host_calls(
                &self.package,
                self.limits,
                self.message_limits,
                &self.initial,
                max_call_bytes,
            )
        } else {
            WasmPluginInstance::activate(
                &self.package,
                self.limits,
                self.message_limits,
                &self.initial,
            )
        };
        let (instance, reply) = activated.map_err(lifecycle_error)?;
        if let Err(error) = decode(&reply, registration) {
            self.lifetime.close();
            let _ = instance.deactivate(&[]);
            return Err(error);
        }
        registration
            .apply_manifest_contracts(&self.package.manifest().compatibility.extension_contracts);
        self.instance.replace(Some(instance));
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        self.deactivate_with_deadline(None)
    }

    fn deactivate_until(&mut self, deadline: Instant) -> Result<(), PluginApiError> {
        self.deactivate_with_deadline(Some(deadline))
    }
}

impl ManagedWasmPlugin {
    fn deactivate_with_deadline(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<(), PluginApiError> {
        let instance = self.instance.take();
        self.lifetime.close();
        if let Some(error) = self.lifetime.fault() {
            return Err(error);
        }
        if let Some(instance) = instance {
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Err(PluginApiError::ExecutionTimedOut);
            }
            instance
                .deactivate_with_deadline(&[], deadline)
                .map_err(lifecycle_error)?;
        }
        Ok(())
    }
}

pub(super) fn lifecycle_error(error: WasmExecutorError) -> PluginApiError {
    match error {
        WasmExecutorError::DeadlineExceeded => PluginApiError::ExecutionTimedOut,
        WasmExecutorError::OutOfFuel
        | WasmExecutorError::GrowthDenied
        | WasmExecutorError::ResponseTooLarge => {
            PluginApiError::ExecutionQuotaExceeded(error.to_string().into())
        }
        WasmExecutorError::Trap(reason) => PluginApiError::ExecutorCrashed(reason.into()),
        WasmExecutorError::InvalidResponseLength => {
            PluginApiError::ExecutorCrashed(error.to_string().into())
        }
        error => PluginApiError::Rejected(error.to_string().into()),
    }
}

impl Drop for ManagedWasmPlugin {
    fn drop(&mut self) {
        self.lifetime.close();
    }
}
