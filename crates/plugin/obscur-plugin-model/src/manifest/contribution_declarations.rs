/// Names of contributions advertised by the manifest.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ContributionDeclarations {
    /// Declared routes.
    #[serde(default)]
    pub routes: Vec<Box<str>>,
    /// Declared screens.
    #[serde(default)]
    pub screens: Vec<Box<str>>,
    /// Declared components.
    #[serde(default)]
    pub components: Vec<Box<str>>,
    /// Declared actions.
    #[serde(default)]
    pub actions: Vec<Box<str>>,
}
