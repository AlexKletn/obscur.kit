//! Architectural layer of a workspace crate and its allowed dependency
//! directions (spec §2.11, §5.9).

/// One architectural layer, derived from a crate's location in the workspace
/// (`crates/<layer>/…` or `tools/…`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Layer {
    /// `crates/foundation/*` — platform-neutral primitives.
    Foundation,
    /// `crates/domain/*` — pure UI domain models and ports.
    Domain,
    /// `crates/application/*` — runtime, headless, testkit.
    Application,
    /// `crates/frontend/*` — IR, CSS, `.okui`, DSL macros.
    Frontend,
    /// `crates/components/*` — primitives, widgets, theme system.
    Components,
    /// `crates/adapters/*` — platform/GPU/IO implementations.
    Adapters,
    /// `crates/diagnostics/*` — devtools protocol and server.
    Diagnostics,
    /// `crates/plugin/*` — plugin model/host/executors.
    Plugin,
    /// `crates/facade/*` — the `obscur-kit` umbrella crate.
    Facade,
    /// `tools/*` — developer tools (xtask, lsp, cli, devtools).
    Tools,
}

impl Layer {
    /// Maps the directory name directly under `crates/` (or the synthetic
    /// `tools` bucket) to a layer.
    pub(crate) fn from_dir(dir: &str) -> Option<Self> {
        Some(match dir {
            "foundation" => Self::Foundation,
            "domain" => Self::Domain,
            "application" => Self::Application,
            "frontend" => Self::Frontend,
            "components" => Self::Components,
            "adapters" => Self::Adapters,
            "diagnostics" => Self::Diagnostics,
            "plugin" => Self::Plugin,
            "facade" => Self::Facade,
            "tools" => Self::Tools,
            _ => return None,
        })
    }

    /// Whether a crate in `self` is allowed to depend on a crate in `target`,
    /// per the allowed dependency directions of spec §2.11 / §5.9.
    ///
    /// Note: the facade (`obscur-kit`) is the apex of the graph and the ban on
    /// non-facade crates depending on it is enforced separately, so no layer
    /// here lists `Facade` as an allowed target.
    pub(crate) fn allows(self, target: Self) -> bool {
        match self {
            // foundation -> foundation only.
            Self::Foundation => matches!(target, Self::Foundation),
            // domain -> foundation + approved lower domain.
            Self::Domain => matches!(target, Self::Foundation | Self::Domain),
            // application -> foundation + domain + frontend + components (+ app).
            Self::Application => matches!(
                target,
                Self::Foundation
                    | Self::Domain
                    | Self::Application
                    | Self::Frontend
                    | Self::Components
            ),
            // frontend -> foundation + domain models (+ sibling frontend crates).
            Self::Frontend => {
                matches!(target, Self::Foundation | Self::Domain | Self::Frontend)
            }
            // components -> foundation + domain (+ sibling components).
            Self::Components => {
                matches!(target, Self::Foundation | Self::Domain | Self::Components)
            }
            // adapter -> foundation + domain port + application (headless) + sibling adapters.
            Self::Adapters => matches!(
                target,
                Self::Foundation | Self::Domain | Self::Application | Self::Adapters
            ),
            // diagnostics -> foundation + domain + application (runtime bridge) + sibling.
            Self::Diagnostics => matches!(
                target,
                Self::Foundation | Self::Domain | Self::Application | Self::Diagnostics
            ),
            // plugin -> foundation + domain + application + frontend (UI IR) + diagnostics + sibling.
            Self::Plugin => matches!(
                target,
                Self::Foundation
                    | Self::Domain
                    | Self::Application
                    | Self::Frontend
                    | Self::Diagnostics
                    | Self::Plugin
            ),
            // facade and developer tools may depend on any selected crate.
            Self::Facade | Self::Tools => true,
        }
    }

    /// Human-readable label for diagnostic messages.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Domain => "domain",
            Self::Application => "application",
            Self::Frontend => "frontend",
            Self::Components => "components",
            Self::Adapters => "adapters",
            Self::Diagnostics => "diagnostics",
            Self::Plugin => "plugin",
            Self::Facade => "facade",
            Self::Tools => "tools",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Layer;

    #[test]
    fn foundation_may_only_depend_on_foundation() {
        assert!(Layer::Foundation.allows(Layer::Foundation));
        assert!(!Layer::Foundation.allows(Layer::Domain));
        assert!(!Layer::Foundation.allows(Layer::Adapters));
    }

    #[test]
    fn domain_may_depend_on_foundation_and_domain_only() {
        assert!(Layer::Domain.allows(Layer::Foundation));
        assert!(Layer::Domain.allows(Layer::Domain));
        assert!(!Layer::Domain.allows(Layer::Application));
        assert!(!Layer::Domain.allows(Layer::Adapters));
    }

    #[test]
    fn adapter_may_reach_application_but_foundation_may_not() {
        assert!(Layer::Adapters.allows(Layer::Application));
        assert!(!Layer::Foundation.allows(Layer::Application));
    }

    #[test]
    fn dir_names_map_to_layers() {
        assert_eq!(Layer::from_dir("foundation"), Some(Layer::Foundation));
        assert_eq!(Layer::from_dir("facade"), Some(Layer::Facade));
        assert_eq!(Layer::from_dir("tools"), Some(Layer::Tools));
        assert_eq!(Layer::from_dir("nonsense"), None);
    }
}
