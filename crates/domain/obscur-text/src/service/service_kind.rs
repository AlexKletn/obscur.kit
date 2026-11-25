/// Which analysis a [`LanguageServiceRequest`](super::LanguageServiceRequest) asks a provider to run
/// (spec §5.13 optional language services).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceKind {
    /// Spellcheck: flag misspelled words.
    Spellcheck,
    /// Grammar: flag grammatical issues.
    Grammar,
    /// Autocorrect: propose replacements for likely typos.
    Autocorrect,
    /// Language detection: identify the text's language.
    LanguageDetection,
}
