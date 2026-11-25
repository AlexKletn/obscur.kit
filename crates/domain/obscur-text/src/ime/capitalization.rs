#[doc = include_str!("../../docs/api/ime/input_hints.doc/Capitalization.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Capitalization {
    /// No automatic capitalization. The enum's own default.
    #[default]
    None,
    /// Capitalize the first letter of each sentence.
    Sentences,
    /// Capitalize the first letter of each word.
    Words,
    /// Capitalize every character.
    Characters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalization_own_default_is_none() {
        assert_eq!(Capitalization::default(), Capitalization::None);
    }
}
