use obscur_runtime::UiRuntime;

pub(super) struct RuntimeTimer {
    pub(super) deadline_nanos: u64,
    pub(super) task: Box<dyn FnOnce(&mut UiRuntime)>,
}
