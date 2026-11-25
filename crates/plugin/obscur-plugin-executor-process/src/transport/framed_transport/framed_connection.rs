use obscur_plugin_host::service::ServiceSession;

pub(super) struct FramedConnection<R, W> {
    pub(super) input: R,
    pub(super) output: W,
    pub(super) session: ServiceSession,
}
