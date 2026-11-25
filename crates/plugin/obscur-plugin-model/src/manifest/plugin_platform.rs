/// A supported host platform.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginPlatform {
    /// Operating system name.
    pub os: Box<str>,
    /// Architecture name.
    pub arch: Box<str>,
}
