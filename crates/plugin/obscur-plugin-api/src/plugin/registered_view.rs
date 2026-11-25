use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    manifest::VersionReq,
};
use obscur_view::view::View;

/// A view registered with an application-defined integration point.
pub struct RegisteredView {
    point: ExtensionPoint,
    definition: ViewContribution,
    manifest_contract: Option<VersionReq>,
    render: Box<dyn Fn() -> View>,
}

impl RegisteredView {
    pub(super) fn new(
        point: ExtensionPoint,
        definition: ViewContribution,
        render: impl Fn() -> View + 'static,
    ) -> Self {
        Self {
            point,
            definition,
            manifest_contract: None,
            render: Box::new(render),
        }
    }

    pub(super) fn set_manifest_contract(&mut self, contract: Option<VersionReq>) {
        self.manifest_contract = contract;
    }

    /// Returns the package manifest requirement for this registration, if present.
    #[must_use]
    pub const fn manifest_contract(&self) -> Option<&VersionReq> {
        self.manifest_contract.as_ref()
    }

    /// Returns the integration point selected by the plugin.
    #[must_use]
    pub const fn point(&self) -> &ExtensionPoint {
        &self.point
    }

    /// Returns the contribution metadata.
    #[must_use]
    pub const fn definition(&self) -> &ViewContribution {
        &self.definition
    }

    /// Invokes the registered render callback.
    #[must_use]
    pub fn render(&self) -> View {
        (self.render)()
    }
}
