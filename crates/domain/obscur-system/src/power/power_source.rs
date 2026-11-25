/// Where the device is currently drawing power (spec §7.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PowerSource {
    /// Mains / wall power — the plugged-in baseline.
    #[default]
    Ac,
    /// Running on battery.
    Battery,
    /// The platform does not report a power source.
    Unknown,
}
