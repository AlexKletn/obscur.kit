#![doc = include_str!("../docs/api/plugins.doc/_module.md")]

pub use obscur_plugin_api::{
    plugin::{PluginApi, PluginRegistration},
    status::PluginApiError,
};
pub use obscur_plugin_host::{
    capability::{
        CapabilityAccessError, CapabilityHandle, CapabilityIssuer, CapabilityRevoker, ServiceScope,
    },
    contribution::MountedView,
    dependency::{DependencyConflict, DependencyGraph, DependencyResolver},
    discovery::{DiscoveredPlugin, PluginDiscovery},
    lifecycle::CrashRecovery,
    manager::{PluginCatalog, PluginFailure, PluginManager},
    service::{
        JsonServiceBridge, JsonServiceRouter, ServiceCallFrame, ServiceCallLimit,
        ServiceDispatchError, ServiceFrameCodec, ServiceHandle, ServicePacketDispatch,
        ServiceRegistry, ServiceRequest, ServiceRevoker, ServiceSession, ServiceSessionError,
        ServiceSessionMetrics, ServiceSessionObserver, ServiceSessionRevoker, ServiceWireError,
    },
    storage::{
        CacheStore, CredentialLease, CredentialScope, PluginStorage, SecureStorageBackend,
        SecureStorageService, StateStore, StorageNamespace, StorageQuota,
    },
    update::{RollbackPlan, StagedHandover, StateMigration, StateMigrationError},
};
pub use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::{ContributionId, PluginId, PluginPrincipal},
    lifecycle::{PluginLifecycleState, ShutdownConcern},
    state::{MigrationDescriptor, StateSchema, StateScope},
};

pub use obscur_plugin_model::manifest::{
    DependencyKind, DependencyManifest, PluginKind, PluginManifest, Version, VersionReq,
};
pub use obscur_plugin_package::{
    archive::{TarPackageReader, ZipPackageReader},
    extraction::{ExtractedEntry, ExtractionPlan, IsolatedDirectory},
    package::{PackageEntry, PackageFormat, PackageReader, PluginPackage},
    path::PackagePath,
    validation::{ManifestError, PackageError, PackageLimits, PackageValidator},
};

#[cfg(any(unix, windows))]
pub use obscur_plugin_package::installation::{
    InstallationStore, InstalledPackage, StagedInstallation,
};

#[cfg(any(unix, windows))]
pub use obscur_plugin_host::storage::FilesystemStateStore;

#[cfg(all(feature = "plugin-native", any(unix, windows)))]
pub use obscur_plugin_executor_native::{
    AbiBuffer, AbiCallResult, AbiHandle, AbiSlice, AbiStatus, HostFunctionTable, NativePlugin,
    NativePluginHandle, NativeServiceSession,
};

#[cfg(feature = "plugin-process")]
pub use obscur_plugin_executor_process::transport::FramedTransport;

#[cfg(all(feature = "plugin-process", any(unix, windows)))]
pub use obscur_plugin_executor_process::executor::{
    ProcessExecutor, ProcessShutdownChannel, ProcessStopHandle, ProcessSupervisor,
    ProcessSupervisorEvent, ProcessWatchdog, ProcessWatchdogOutcome,
};

#[cfg(feature = "plugin-wasm")]
pub use obscur_plugin_executor_wasm::{
    error::WasmExecutorError,
    executor::{
        ManagedWasmHandle, ManagedWasmPlugin, WasmExecutor, WasmLimits, WasmMessageLimits,
        WasmPluginInstance,
    },
};

#[cfg(feature = "plugin-wasm")]
pub use obscur_plugin_host::service::{WasmServiceDriver, WasmServiceDriverError};

mod plugin_ui;
pub use plugin_ui::PluginUi;
