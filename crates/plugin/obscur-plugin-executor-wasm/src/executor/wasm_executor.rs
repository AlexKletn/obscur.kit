//! Bounded execution of verified WebAssembly modules.

use std::time::Instant;

use obscur_plugin_model::manifest::PluginKind;
use obscur_plugin_package::package::PluginPackage;
use wasmi::{
    Config, EnforcedLimits, Engine, Instance, Linker, Module, Store, StoreLimitsBuilder, TrapCode,
    TypedFunc, TypedResumableCall,
};

use super::host_call::{self, WasmState};
use crate::error::WasmExecutorError;

use super::{WasmLimits, WasmMessageLimits};

/// Owns one isolated WASM instance with host-defined memory, table, and execution budgets.
pub struct WasmExecutor {
    store: Store<WasmState>,
    instance: Instance,
    fuel_per_call: u64,
    faulted: bool,
}

impl WasmExecutor {
    /// Instantiates the verified entrypoint of a WASM package.
    ///
    /// The runtime accepts binary modules with no imports or start function.
    /// Each instance has at most one linear memory and one table.
    pub fn load(package: &PluginPackage, limits: WasmLimits) -> Result<Self, WasmExecutorError> {
        Self::load_configured(package, limits, None)
    }

    /// Enables the bounded `obscur_host.submit_v1` guest import.
    pub fn load_with_host_calls(
        package: &PluginPackage,
        limits: WasmLimits,
        max_call_bytes: usize,
    ) -> Result<Self, WasmExecutorError> {
        Self::load_configured(package, limits, Some(max_call_bytes))
    }

    fn load_configured(
        package: &PluginPackage,
        limits: WasmLimits,
        max_call_bytes: Option<usize>,
    ) -> Result<Self, WasmExecutorError> {
        if package.manifest().kind != PluginKind::Wasm {
            return Err(WasmExecutorError::WrongPluginKind);
        }
        let bytes = package.entrypoint().bytes();
        if bytes.len() > limits.module_bytes {
            return Err(WasmExecutorError::ModuleTooLarge);
        }

        if !bytes.starts_with(b"\0asm") {
            return Err(WasmExecutorError::InvalidModule(
                "missing binary WASM header".into(),
            ));
        }

        let mut config = Config::default();
        config
            .consume_fuel(true)
            .allow_start_fn(false)
            .ignore_custom_sections(true)
            .enforced_limits(EnforcedLimits::strict());
        let engine = Engine::new(&config);
        let module = Module::new(&engine, bytes)
            .map_err(|error| WasmExecutorError::InvalidModule(error.to_string()))?;
        if module.imports().any(|import| {
            max_call_bytes.is_none()
                || import.module() != host_call::MODULE
                || import.name() != host_call::SUBMIT
        }) {
            return Err(WasmExecutorError::ImportsUnsupported);
        }

        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.memory_bytes)
            .table_elements(limits.table_elements)
            .instances(1)
            .memories(1)
            .tables(1)
            .trap_on_grow_failure(true)
            .build();
        let mut store = Store::new(
            &engine,
            WasmState {
                limits: store_limits,
                max_call_bytes,
                pending_call: None,
            },
        );
        store.limiter(|state| &mut state.limits);
        let mut linker = Linker::new(&engine);
        if max_call_bytes.is_some() {
            linker
                .func_wrap(host_call::MODULE, host_call::SUBMIT, host_call::submit)
                .map_err(|error| WasmExecutorError::Instantiation(error.to_string()))?;
        }
        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|error| WasmExecutorError::Instantiation(error.to_string()))?;
        Ok(Self {
            store,
            instance,
            fuel_per_call: limits.fuel_per_call,
            faulted: false,
        })
    }

    /// Removes the packet submitted during the last successful guest invocation.
    #[must_use]
    pub fn take_host_call(&mut self) -> Option<Vec<u8>> {
        self.store.data_mut().pending_call.take()
    }

    fn ensure_ready(&self) -> Result<(), WasmExecutorError> {
        if self.faulted {
            Err(WasmExecutorError::Faulted)
        } else if self.store.data().pending_call.is_some() {
            Err(WasmExecutorError::HostCallPending)
        } else {
            Ok(())
        }
    }

    fn fault(&mut self) {
        self.faulted = true;
        self.store.data_mut().pending_call = None;
    }

    /// Invokes a host-selected export with signature `(i32) -> i32`.
    ///
    /// A trap faults the instance. The host can load the validated package
    /// again to start with fresh state.
    pub fn call_i32(&mut self, export: &str, value: i32) -> Result<i32, WasmExecutorError> {
        self.ensure_ready()?;
        let function = self.export_i32(export)?;
        self.call_function(function, value)
    }

    /// Exchanges bytes with the guest through the version-1 message buffer.
    ///
    /// The guest exports `memory`, two i32 globals
    /// `obscur_plugin_buffer_ptr` and `obscur_plugin_buffer_capacity`, and
    /// `obscur_plugin_dispatch_v1(i32) -> i32`. The function reads the input
    /// from that buffer, writes its output there, and returns the reply length.
    pub fn exchange(
        &mut self,
        request: &[u8],
        limits: WasmMessageLimits,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        self.exchange_with("obscur_plugin_dispatch_v1", request, limits)
    }

    pub(super) fn exchange_with(
        &mut self,
        export: &str,
        request: &[u8],
        limits: WasmMessageLimits,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        self.exchange_with_deadline(export, request, limits, None)
    }

    pub(super) fn exchange_with_deadline(
        &mut self,
        export: &str,
        request: &[u8],
        limits: WasmMessageLimits,
        deadline: Option<Instant>,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        self.ensure_ready()?;
        if request.len() > limits.request_bytes {
            return Err(WasmExecutorError::RequestTooLarge);
        }
        let function = self.export_i32(export)?;
        let memory = self
            .instance
            .get_memory(&self.store, "memory")
            .ok_or(WasmExecutorError::InvalidMessageBuffer)?;
        let offset = self.buffer_global("obscur_plugin_buffer_ptr")?;
        let capacity = self.buffer_global("obscur_plugin_buffer_capacity")?;
        if offset
            .checked_add(capacity)
            .is_none_or(|end| end > memory.data_size(&self.store))
        {
            return Err(WasmExecutorError::InvalidMessageBuffer);
        }
        if request.len() > capacity {
            return Err(WasmExecutorError::RequestTooLarge);
        }
        let request_size =
            i32::try_from(request.len()).map_err(|_| WasmExecutorError::RequestTooLarge)?;
        memory
            .write(&mut self.store, offset, request)
            .map_err(|_| WasmExecutorError::InvalidMessageBuffer)?;

        let reply = if let Some(deadline) = deadline {
            self.call_until(function, request_size, deadline)
        } else {
            self.call_function(function, request_size)
        }?;
        let size = match usize::try_from(reply) {
            Ok(size) if size <= capacity => size,
            _ => {
                self.fault();
                return Err(WasmExecutorError::InvalidResponseLength);
            }
        };
        if size > limits.response_bytes {
            self.fault();
            return Err(WasmExecutorError::ResponseTooLarge);
        }
        let mut response = vec![0; size];
        memory
            .read(&self.store, offset, &mut response)
            .map_err(|_| WasmExecutorError::InvalidMessageBuffer)?;
        Ok(response)
    }

    fn buffer_global(&self, name: &str) -> Result<usize, WasmExecutorError> {
        self.instance
            .get_global(&self.store, name)
            .and_then(|global| global.get(&self.store).i32())
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(WasmExecutorError::InvalidMessageBuffer)
    }

    pub(super) fn has_export(&self, name: &str) -> bool {
        self.instance.get_export(&self.store, name).is_some()
    }

    pub(super) fn export_i32(
        &self,
        export: &str,
    ) -> Result<TypedFunc<i32, i32>, WasmExecutorError> {
        self.instance
            .get_typed_func::<i32, i32>(&self.store, export)
            .map_err(|_| WasmExecutorError::InvalidExport)
    }

    fn call_function(
        &mut self,
        function: TypedFunc<i32, i32>,
        value: i32,
    ) -> Result<i32, WasmExecutorError> {
        self.store
            .set_fuel(self.fuel_per_call)
            .map_err(|error| WasmExecutorError::Trap(error.to_string()))?;
        function.call(&mut self.store, value).map_err(|error| {
            self.fault();
            execution_error(&error)
        })
    }

    fn call_until(
        &mut self,
        function: TypedFunc<i32, i32>,
        value: i32,
        deadline: Instant,
    ) -> Result<i32, WasmExecutorError> {
        const FUEL_SLICE: u64 = 10_000;
        let result = (|| {
            if Instant::now() >= deadline {
                return Err(WasmExecutorError::DeadlineExceeded);
            }
            let mut fuel = self.fuel_per_call.min(FUEL_SLICE);
            let mut reserve = self.fuel_per_call - fuel;
            self.store
                .set_fuel(fuel)
                .map_err(|error| execution_error(&error))?;
            let mut call = function.call_resumable(&mut self.store, value);
            loop {
                if Instant::now() >= deadline {
                    return Err(WasmExecutorError::DeadlineExceeded);
                }
                match call.map_err(|error| execution_error(&error))? {
                    TypedResumableCall::Finished(value) => return Ok(value),
                    TypedResumableCall::HostTrap(trap) => {
                        return Err(execution_error(trap.host_error()));
                    }
                    TypedResumableCall::OutOfFuel(suspended) => {
                        reserve += self
                            .store
                            .get_fuel()
                            .map_err(|error| execution_error(&error))?;
                        if reserve < suspended.required_fuel() {
                            return Err(WasmExecutorError::OutOfFuel);
                        }
                        fuel = reserve.min(FUEL_SLICE.max(suspended.required_fuel()));
                        reserve -= fuel;
                        self.store
                            .set_fuel(fuel)
                            .map_err(|error| execution_error(&error))?;
                        call = suspended.resume(&mut self.store);
                    }
                }
            }
        })();
        if result.is_err() {
            self.fault();
        }
        result
    }
}

fn execution_error(error: &wasmi::Error) -> WasmExecutorError {
    match error.as_trap_code() {
        Some(TrapCode::OutOfFuel) => WasmExecutorError::OutOfFuel,
        Some(TrapCode::GrowthOperationLimited) => WasmExecutorError::GrowthDenied,
        _ => WasmExecutorError::Trap(error.to_string()),
    }
}
