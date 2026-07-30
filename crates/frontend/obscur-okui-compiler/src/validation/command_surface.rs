//! `CommandSurface`: the set of command names a document is allowed to reference
//! from an `on:<event>` binding (spec §2.13).

/// The commands a document may bind a handler to — the declared side of §2.13's
/// "Unknown attributes, events and **string method names** are compile errors."
///
/// # What is being checked, and why nothing checked it
///
/// `on:click=increment` names a handler by string. The front ends hash that string
/// (`name_binding::fallback_symbol`, pinned equal to `InternedSymbol::named` by a
/// drift test) into the command symbol the runtime dispatches on, and the runtime
/// matches it against whatever an application registered. The hashing is sound;
/// what was missing is that **nothing ever asked whether the name is claimed by
/// anybody**. A misspelled handler was a silent no-op: no compile error, no
/// runtime diagnostic, a button that does nothing. That is precisely the failure
/// §2.13's sentence exists to prevent, and it is the one member of its list —
/// attributes, events, string method names — that was unenforced.
///
/// # Why the declaration comes from the caller
///
/// A command is claimed by a *runtime registration*
/// (`UiRuntime::on_command`/`on_command_at`, `App::on_command`), which is a value
/// that exists when the application runs — after every compiler has finished. No
/// pass over the document can discover it, so the checkable form of the rule is
/// the one §2.13 already uses for capabilities: the **caller declares** what
/// exists, and the compiler holds the document to it.
/// [`Compiler::granting`](crate::compile::Compiler::granting) does exactly this for
/// `requires="camera"`; [`Compiler::claiming_command`](crate::compile::Compiler::claiming_command)
/// does it for handlers.
///
/// # Why the default is [`Unchecked`](Self::Unchecked) and not an empty set
///
/// Because a false "unknown command" is worse than a silent no-op. A document
/// carries no declaration of its own — §2.13 names no authoring surface for one,
/// and inventing syntax the spec does not describe is not this type's business —
/// so a caller that declares nothing has told the compiler nothing, which is not
/// the same as telling it that no command exists. The two states are therefore
/// named rather than distinguished by emptiness:
/// [`Declared`](Self::Declared)`(vec![])` means "this document may bind no handler
/// at all", and that is a real, checkable claim a caller can make.
///
/// # The half that stays unreachable
///
/// The `okui!` macro cannot use this, and no wiring will change that: it expands
/// while the application crate is still compiling, so the registrations it would
/// check against do not exist yet, and a proc-macro has no argument to be handed a
/// surface through. See
/// [`ContractRegistry::register_surface`](crate::compile::ContractRegistry::register_surface),
/// whose reachability analysis is the same one, component-for-command.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CommandSurface {
    /// No command names were declared, so no handler reference can be judged. The
    /// default: silence about the surface must not read as a claim about it.
    #[default]
    Unchecked,
    /// Exactly these command names are claimed. A handler naming anything else is
    /// a compile error — including when the list is empty, which claims that the
    /// document binds no handler at all.
    Declared(Vec<Box<str>>),
}

impl CommandSurface {
    /// A closed surface claiming exactly `names`.
    #[must_use]
    pub fn declared<N: Into<Box<str>>>(names: impl IntoIterator<Item = N>) -> Self {
        Self::Declared(names.into_iter().map(Into::into).collect())
    }

    /// Adds `name` to the claimed commands, closing the surface if it was
    /// [`Unchecked`](Self::Unchecked).
    ///
    /// Naming one command is what turns the check on: a caller that can name one
    /// registration can name them all, and a partial list would report the rest as
    /// unknown.
    pub fn claim(&mut self, name: impl Into<Box<str>>) {
        match self {
            Self::Declared(names) => names.push(name.into()),
            Self::Unchecked => *self = Self::Declared(vec![name.into()]),
        }
    }

    /// Returns whether this surface can judge a handler reference at all.
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self, Self::Declared(_))
    }

    /// Returns the declared command names — empty for
    /// [`Unchecked`](Self::Unchecked), which is *not* a claim that no command
    /// exists. Ask [`is_checked`](Self::is_checked) before reading emptiness.
    #[must_use]
    pub fn names(&self) -> &[Box<str>] {
        match self {
            Self::Unchecked => &[],
            Self::Declared(names) => names,
        }
    }

    /// Returns whether `name` is claimed by a declared command.
    ///
    /// Always `true` on an [`Unchecked`](Self::Unchecked) surface: with nothing
    /// declared, no name can be shown to be unclaimed, and refusing one would be
    /// the false error this type exists to avoid.
    #[must_use]
    pub fn claims(&self, name: &str) -> bool {
        match self {
            Self::Unchecked => true,
            Self::Declared(names) => names.iter().any(|claimed| claimed.as_ref() == name),
        }
    }

    /// The declared command name closest to `name`, for a did-you-mean suggestion,
    /// or [`None`] when nothing is close enough.
    ///
    /// # The bar
    ///
    /// Levenshtein distance, with an explicit threshold — 1 for a name of three
    /// characters or fewer, 2 above that — because a suggestion that is not
    /// obviously the intended name is noise, and short names are too easy to be
    /// two edits from each other by coincidence (`add` and `sub` would suggest each
    /// other at 2). The nearest candidate wins; a tie goes to the first declared,
    /// so the answer is deterministic in declaration order rather than in hash
    /// order.
    ///
    /// A spec test recorded this as a candidate amendment: §2.13 asks for the
    /// compile error and says nothing about suggestions, so the bar is an
    /// implementation choice stated here rather than a requirement met.
    #[must_use]
    pub fn nearest(&self, name: &str) -> Option<&str> {
        let limit = if name.chars().count() <= 3 { 1 } else { 2 };
        self.names()
            .iter()
            .map(|claimed| (edit_distance(claimed, name), claimed))
            .filter(|(distance, _)| *distance <= limit)
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, claimed)| claimed.as_ref())
    }
}

/// The Levenshtein distance between `left` and `right`, counted over `char`s.
///
/// The textbook two-row dynamic program: `O(len × len)` time, `O(len)` space, over
/// characters rather than bytes so a non-ASCII handler name is not measured in
/// UTF-8 code units.
fn edit_distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (row, left_char) in left.chars().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right.iter().enumerate() {
            let substitution = usize::from(left_char != *right_char);
            current[column + 1] = (previous[column] + substitution)
                .min(previous[column + 1] + 1)
                .min(current[column] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unchecked_surface_judges_nothing() {
        // The default has to be silent: every document written before commands
        // could be declared still compiles, and a caller that declares nothing has
        // made no claim to hold anyone to.
        let surface = CommandSurface::default();
        assert!(!surface.is_checked());
        assert!(surface.claims("anything"));
        assert!(surface.nearest("anything").is_none());
    }

    #[test]
    fn a_declared_surface_admits_what_it_claims_and_nothing_else() {
        let surface = CommandSurface::declared(["increment", "reset"]);
        assert!(surface.is_checked());
        assert!(surface.claims("increment"));
        assert!(!surface.claims("incrment"));
    }

    #[test]
    fn an_empty_declaration_is_a_claim_and_not_a_default() {
        // The state emptiness cannot express when it doubles as "unset": a document
        // that binds no handler at all.
        let surface = CommandSurface::declared(Vec::<&str>::new());
        assert!(surface.is_checked());
        assert!(!surface.claims("increment"));
    }

    #[test]
    fn claiming_one_command_closes_the_surface() {
        let mut surface = CommandSurface::Unchecked;
        surface.claim("increment");
        assert_eq!(surface, CommandSurface::declared(["increment"]));
        surface.claim("reset");
        assert_eq!(surface.names().len(), 2);
    }

    #[test]
    fn the_nearest_name_is_suggested_within_the_bar() {
        let surface = CommandSurface::declared(["increment", "decrement"]);
        assert_eq!(surface.nearest("incrment"), Some("increment"));
        assert_eq!(
            surface.nearest("increment"),
            Some("increment"),
            "an exact match is trivially nearest"
        );
        assert_eq!(
            surface.nearest("totally_different"),
            None,
            "nothing within two edits, so no misleading guess"
        );
    }

    #[test]
    fn a_short_name_is_held_to_a_tighter_bar() {
        // `add` and `sub` are two edits apart and mean opposite things; suggesting
        // one for the other would be worse than saying nothing.
        let surface = CommandSurface::declared(["add"]);
        assert_eq!(surface.nearest("sub"), None);
        assert_eq!(surface.nearest("ad"), Some("add"));
    }

    #[test]
    fn edit_distance_counts_characters_not_bytes() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "abd"), 1);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        // One 2-byte character replaced by another: one edit, not two.
        assert_eq!(edit_distance("привет", "приват"), 1);
    }
}
