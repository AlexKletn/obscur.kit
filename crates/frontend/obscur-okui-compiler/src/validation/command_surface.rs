#![doc = include_str!("command_surface.doc/_module.md")]

#[doc = include_str!("command_surface.doc/CommandSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CommandSurface {
    #[doc = include_str!("command_surface.doc/Unchecked.md")]
    #[default]
    Unchecked,
    #[doc = include_str!("command_surface.doc/Declared.md")]
    Declared(Vec<Box<str>>),
}

impl CommandSurface {
    #[doc = include_str!("command_surface.doc/declared.md")]
    #[must_use]
    pub fn declared<N: Into<Box<str>>>(names: impl IntoIterator<Item = N>) -> Self {
        Self::Declared(names.into_iter().map(Into::into).collect())
    }

    #[doc = include_str!("command_surface.doc/claim.md")]
    pub fn claim(&mut self, name: impl Into<Box<str>>) {
        match self {
            Self::Declared(names) => names.push(name.into()),
            Self::Unchecked => *self = Self::Declared(vec![name.into()]),
        }
    }

    #[doc = include_str!("command_surface.doc/is_checked.md")]
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self, Self::Declared(_))
    }

    #[doc = include_str!("command_surface.doc/names.md")]
    #[must_use]
    pub fn names(&self) -> &[Box<str>] {
        match self {
            Self::Unchecked => &[],
            Self::Declared(names) => names,
        }
    }

    #[doc = include_str!("command_surface.doc/claims.md")]
    #[must_use]
    pub fn claims(&self, name: &str) -> bool {
        match self {
            Self::Unchecked => true,
            Self::Declared(names) => names.iter().any(|claimed| claimed.as_ref() == name),
        }
    }

    #[doc = include_str!("command_surface.doc/nearest.md")]
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

#[doc = include_str!("command_surface.doc/edit_distance.md")]
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
