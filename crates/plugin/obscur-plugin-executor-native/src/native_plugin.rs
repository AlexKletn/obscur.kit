//! Loads a verified native package and drives its versioned lifecycle.

use std::{cell::RefCell, mem::ManuallyDrop, path::Path, rc::Rc, time::Instant};

use super::{
    NativePluginHandle, NativeServiceSession,
    native_callback_status::check_status,
    native_dispatch_state::{NativeDispatch, NativeDispatchState},
};
use libloading::Library;
use obscur_plugin_abi::{
    function_table::{HostFunctionTable, PluginFunctionTable},
    value::{AbiHandle, AbiStatus},
};
use obscur_plugin_api::{
    plugin::{PluginApi, PluginRegistration},
    status::PluginApiError,
};
use obscur_plugin_model::{
    identity::PluginId,
    manifest::{PluginKind, Version},
};
use obscur_plugin_package::{
    extraction::{ExtractionPlan, IsolatedDirectory},
    package::PluginPackage,
};

type LifecycleCallback = extern "C" fn(AbiHandle) -> AbiStatus;
type BindCallback = extern "C" fn(AbiHandle, HostFunctionTable) -> AbiStatus;

mod activation;
use activation::Activation;

/// One logical instance of a trusted native plugin. Its library remains loaded until process exit.
pub struct NativePlugin {
    id: PluginId,
    version: Version,
    handle: AbiHandle,
    initialize: LifecycleCallback,
    deactivate: LifecycleCallback,
    activation: Activation,
    host_binding: Option<(HostFunctionTable, BindCallback)>,
    dispatch: Rc<RefCell<NativeDispatchState>>,
    _extraction: IsolatedDirectory,
}

impl NativePlugin {
    #[doc = include_str!("../docs/api/native_plugin.doc/load.md")]
    pub unsafe fn load(
        package: &PluginPackage,
        extraction_parent: &Path,
        handle: AbiHandle,
    ) -> Result<Self, PluginApiError> {
        // SAFETY: the public loader requires a trusted version-1 library.
        unsafe { Self::load_inner(package, extraction_parent, handle, None) }
    }

    #[doc = include_str!("../docs/api/native_plugin.doc/load_with_host.md")]
    pub unsafe fn load_with_host(
        package: &PluginPackage,
        extraction_parent: &Path,
        handle: AbiHandle,
        host: HostFunctionTable,
    ) -> Result<Self, PluginApiError> {
        if !host.supports_v1() {
            return Err(PluginApiError::Rejected(
                "incompatible native host table".into(),
            ));
        }
        // SAFETY: the public loader requires compatible plugin and host callbacks.
        unsafe { Self::load_inner(package, extraction_parent, handle, Some(host)) }
    }

    #[doc = include_str!("../docs/api/native_plugin.doc/load_with_services.md")]
    pub unsafe fn load_with_services(
        package: &PluginPackage,
        extraction_parent: &Path,
        handle: AbiHandle,
        services: NativeServiceSession,
    ) -> Result<Self, PluginApiError> {
        if services.principal().plugin() != &package.manifest().id {
            return Err(PluginApiError::Rejected(
                "native service session principal mismatch".into(),
            ));
        }
        // SAFETY: the caller approves the native package and its binding ABI.
        let plugin = unsafe {
            Self::load_with_host(
                package,
                extraction_parent,
                handle,
                NativeServiceSession::table(),
            )
        }?;
        plugin.dispatch.borrow_mut().services = Some(services);
        Ok(plugin)
    }

    unsafe fn load_inner(
        package: &PluginPackage,
        extraction_parent: &Path,
        handle: AbiHandle,
        host: Option<HostFunctionTable>,
    ) -> Result<Self, PluginApiError> {
        if package.manifest().kind != PluginKind::Native {
            return Err(PluginApiError::Rejected(
                "package is not a native plugin".into(),
            ));
        }
        let extraction = ExtractionPlan::new(package)
            .extract_in(extraction_parent)
            .map_err(|error| load_failure("extract package", error))?;
        // SAFETY: the caller approved the exact validated binary and its dependencies.
        let library = unsafe { open_library(extraction.entrypoint().path()) }
            .map_err(|error| load_failure("open library", error))?;
        // Constructors may have run. Keep the image loaded even if negotiation fails.
        let library = ManuallyDrop::new(library);
        // SAFETY: the caller guarantees that this symbol uses the version-1 ABI.
        let entry = unsafe {
            library.get::<unsafe extern "C" fn() -> PluginFunctionTable>(
                PluginFunctionTable::ENTRYPOINT.as_bytes(),
            )
        }
        .map_err(|error| load_failure("resolve lifecycle entry point", error))?;
        // SAFETY: the entry point has the trusted ABI and library lifetime is pinned.
        let table = unsafe { entry() };
        let (Some(initialize), Some(deactivate)) = (table.initialize, table.deactivate) else {
            return Err(PluginApiError::Rejected(
                "native lifecycle callbacks are missing".into(),
            ));
        };
        if !table.supports_v1() {
            return Err(PluginApiError::Rejected(
                "incompatible native lifecycle table".into(),
            ));
        }
        let host_binding = if let Some(host) = host {
            // SAFETY: the caller trusts the plugin symbol to implement this ABI.
            let bind = unsafe {
                library.get::<BindCallback>(HostFunctionTable::BIND_ENTRYPOINT.as_bytes())
            }
            .map_err(|error| load_failure("resolve host binding entry point", error))?;
            Some((host, *bind))
        } else {
            None
        };
        // SAFETY: the trusted library implements the documented ABI for any exported optional symbol.
        let dispatch = unsafe {
            library.get::<NativeDispatch>(PluginFunctionTable::DISPATCH_ENTRYPOINT.as_bytes())
        }
        .ok()
        .map(|callback| *callback);
        Ok(Self {
            id: package.manifest().id.clone(),
            version: package.manifest().version.clone(),
            handle,
            initialize,
            deactivate,
            activation: Activation::default(),
            host_binding,
            dispatch: Rc::new(RefCell::new(NativeDispatchState {
                active: false,
                dispatch,
                services: None,
            })),
            _extraction: extraction,
        })
    }

    /// Returns the ID from the verified package.
    #[must_use]
    pub const fn id(&self) -> &PluginId {
        &self.id
    }

    /// Returns the version from the verified package.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns a non-owning message handle that becomes usable after activation.
    #[must_use]
    pub fn handle(&self) -> NativePluginHandle {
        NativePluginHandle {
            handle: self.handle,
            state: Rc::downgrade(&self.dispatch),
        }
    }
}

impl PluginApi for NativePlugin {
    fn suspend(&mut self) {
        self.dispatch.borrow_mut().active = false;
    }

    fn resume(&mut self) {
        self.dispatch.borrow_mut().active = self.activation.is_ready();
    }

    fn initialize(&mut self, _: &mut PluginRegistration) -> Result<(), PluginApiError> {
        let initialize = || {
            if let Some((host, bind)) = self.host_binding {
                check_status("bind host", bind(self.handle, host))?;
            }
            check_status("initialize", (self.initialize)(self.handle))
        };
        let mut state = self.dispatch.borrow_mut();
        let result = self.activation.initialize(|| match &state.services {
            Some(services) => services.enter(self.handle, initialize),
            None => initialize(),
        });
        if result.is_ok() {
            state.active = true;
        }
        result
    }

    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        self.close_dispatch()?;
        if self.activation.take_cleanup() {
            check_status("deactivate", (self.deactivate)(self.handle))?;
        }
        Ok(())
    }

    fn deactivate_until(&mut self, deadline: Instant) -> Result<(), PluginApiError> {
        if Instant::now() >= deadline {
            self.close_dispatch()?;
            self.activation.take_cleanup();
            return Err(PluginApiError::ExecutionTimedOut);
        }
        self.deactivate()
    }
}

impl NativePlugin {
    fn close_dispatch(&mut self) -> Result<(), PluginApiError> {
        let mut state = self
            .dispatch
            .try_borrow_mut()
            .map_err(|_| PluginApiError::Rejected("native plugin is busy".into()))?;
        state.active = false;
        state.services = None;
        Ok(())
    }
}

impl Drop for NativePlugin {
    fn drop(&mut self) {
        let _ = self.deactivate();
    }
}

fn load_failure(operation: &str, error: impl std::fmt::Display) -> PluginApiError {
    PluginApiError::Rejected(format!("could not {operation} for native plugin: {error}").into())
}

#[cfg(unix)]
unsafe fn open_library(path: &Path) -> Result<Library, libloading::Error> {
    // SAFETY: the native loader's trust requirements apply to this path.
    unsafe { Library::new(path) }
}

#[cfg(windows)]
unsafe fn open_library(path: &Path) -> Result<Library, libloading::Error> {
    use libloading::os::windows::{
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, Library as WindowsLibrary,
    };

    // SAFETY: the native loader's trust requirements apply to this path.
    unsafe {
        WindowsLibrary::load_with_flags(
            path,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
        .map(Into::into)
    }
}
