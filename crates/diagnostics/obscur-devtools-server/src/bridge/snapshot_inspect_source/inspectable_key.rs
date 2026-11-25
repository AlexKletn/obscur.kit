use obscur_devtools_protocol::message::Inspectable;

/// A hashable/orderable key for [`Inspectable`], which is `Copy` + `Eq` but not
/// `Ord`; keying a [`BTreeMap`](std::collections::BTreeMap) on its stable discriminant avoids requiring the
/// protocol enum to derive `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct InspectableKey(u8);

impl From<Inspectable> for InspectableKey {
    fn from(target: Inspectable) -> Self {
        Self(match target {
            Inspectable::Tree => 0,
            Inspectable::Styles => 1,
            Inspectable::Layout => 2,
            Inspectable::EventFlow => 3,
            Inspectable::Focus => 4,
            Inspectable::Animations => 5,
            Inspectable::Resources => 6,
            Inspectable::Accessibility => 7,
            Inspectable::Scene => 8,
            Inspectable::Rendering => 9,
            Inspectable::Caches => 10,
            Inspectable::PluginState => 11,
        })
    }
}
