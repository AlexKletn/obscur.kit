#[doc = include_str!("../../docs/api/presentation/sleep_inhibition.doc/SleepInhibitionScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SleepInhibitionScope {
    /// Inhibit full system sleep, keeping the machine awake.
    System,
    /// Inhibit only display sleep and the screensaver.
    Display,
}
