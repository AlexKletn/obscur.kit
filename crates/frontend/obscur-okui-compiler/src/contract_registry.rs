use std::collections::HashMap;

use obscur_view::contract::ComponentSurface;

use crate::validation::{ComponentContract, default_contracts};

#[doc = include_str!("../docs/api/compile.doc/ContractRegistry.md")]
#[derive(Debug, Clone, Default)]
pub struct ContractRegistry {
    contracts: HashMap<Box<str>, ComponentContract>,
}

impl ContractRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../docs/api/compile.doc/builtins.md")]
    #[must_use]
    pub fn builtins() -> Self {
        let mut registry = Self::new();
        for contract in default_contracts() {
            registry.register(contract);
        }
        registry
    }

    /// Registers `contract` (keyed by its component name) and returns the
    /// registry (fluent).
    #[must_use]
    pub fn with(mut self, contract: ComponentContract) -> Self {
        self.register(contract);
        self
    }

    /// Registers `contract`, returning the contract it displaced for the same
    /// component name, if any.
    pub fn register(&mut self, contract: ComponentContract) -> Option<ComponentContract> {
        self.contracts.insert(contract.component().into(), contract)
    }

    #[doc = include_str!("../docs/api/compile.doc/register_surface.md")]
    pub fn register_surface(&mut self, surface: &ComponentSurface) -> Option<ComponentContract> {
        self.register(surface.into())
    }

    /// Registers a user component's [`ComponentSurface`] and returns the
    /// registry (fluent); see [`register_surface`](Self::register_surface).
    #[must_use]
    pub fn with_surface(mut self, surface: &ComponentSurface) -> Self {
        self.register_surface(surface);
        self
    }

    /// Returns the contract registered for `component`, if any.
    #[must_use]
    pub fn get(&self, component: &str) -> Option<&ComponentContract> {
        self.contracts.get(component)
    }

    /// Returns the number of registered contracts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    /// Returns whether no contracts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }
}
