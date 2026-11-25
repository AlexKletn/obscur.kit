//! A diagnostic explaining why a plugin inventory cannot be activated.

use std::{error::Error, fmt};

use obscur_plugin_model::{
    identity::PluginId,
    manifest::{DependencyManifest, Version},
};

/// A diagnostic explaining why a plugin inventory cannot be activated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyConflict {
    /// The inventory contains more than one manifest with this ID.
    DuplicatePlugin(PluginId),
    /// A consumer declares the same provider more than once.
    DuplicateDependency {
        /// Plugin declaring the dependency.
        consumer: PluginId,
        /// Provider declared more than once.
        provider: PluginId,
    },
    /// A required or service-only provider is absent.
    MissingProvider {
        /// Plugin declaring the dependency.
        consumer: PluginId,
        /// Unsatisfied dependency.
        dependency: DependencyManifest,
    },
    /// The inventory contains a provider outside the requested version range.
    IncompatibleVersion {
        /// Plugin declaring the dependency.
        consumer: PluginId,
        /// Unsatisfied dependency.
        dependency: DependencyManifest,
        /// Provider version in the inventory.
        found: Version,
    },
    /// A closed chain of dependencies, with the first ID repeated at the end.
    Cycle(Vec<PluginId>),
}

impl fmt::Display for DependencyConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePlugin(id) => write!(f, "multiple manifests for plugin {}", id.as_str()),
            Self::DuplicateDependency { consumer, provider } => write!(
                f,
                "plugin {} declares dependency {} more than once",
                consumer.as_str(),
                provider.as_str(),
            ),
            Self::MissingProvider {
                consumer,
                dependency,
            } => write!(
                f,
                "plugin {} requires {} {} ({:?}), but the provider is missing",
                consumer.as_str(),
                dependency.id.as_str(),
                dependency.version,
                dependency.kind,
            ),
            Self::IncompatibleVersion {
                consumer,
                dependency,
                found,
            } => write!(
                f,
                "plugin {} requests {} {} ({:?}), but the provider version is {}",
                consumer.as_str(),
                dependency.id.as_str(),
                dependency.version,
                dependency.kind,
                found,
            ),
            Self::Cycle(path) => {
                f.write_str("plugin dependency cycle: ")?;
                for (index, id) in path.iter().enumerate() {
                    if index != 0 {
                        f.write_str(" -> ")?;
                    }
                    f.write_str(id.as_str())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for DependencyConflict {}
