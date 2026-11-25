/// The kinds of thing that can be wrong with path data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathDataFault {
    /// A letter that names no command.
    UnknownCommand,
    /// A command whose numbers ran out.
    MissingNumber,
    #[doc = include_str!("../../docs/api/path/path_data_fault.doc/LeadingCommand.md")]
    LeadingCommand,
    #[doc = include_str!("../../docs/api/path/path_data_fault.doc/StrayNumber.md")]
    StrayNumber,
    /// An elliptical arc flag is missing or is not a single 0 or 1.
    InvalidArcFlag,
    /// A path coordinate cannot be represented after applying its origin or reflection.
    CoordinateOutOfRange,
    /// The arc requires a point beyond the representable finite path coordinates.
    ArcOutOfRange,
}
