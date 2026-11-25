use obscur_plugin_model::identity::PluginId;

use crate::contribution::MountedView;
use crate::manager::PluginRecord;

pub(super) struct PreviousPlugin {
    pub(super) id: PluginId,
    pub(super) record: PluginRecord,
    pub(super) pages: Vec<MountedView>,
}
