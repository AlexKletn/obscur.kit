use obscur_plugin_model::identity::PluginPrincipal;

/// A typed service call with caller identity assigned by the host.
#[derive(Debug)]
pub struct ServiceRequest<T> {
    /// The authenticated principal of the caller.
    pub caller: PluginPrincipal,
    /// The operation authorized by the handle.
    pub operation: Box<str>,
    /// The request payload for the service contract.
    pub payload: T,
}
