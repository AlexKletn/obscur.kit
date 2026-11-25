/// The one necessary condition a selector's subject compound states about the
/// element it can match, if it states one at all.
pub(super) enum SubjectKey<'a> {
    Id(&'a str),
    Class(&'a str),
    Type(&'a str),
}
