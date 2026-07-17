//! `MessageArgs`: the named parameters a message is formatted with (spec §7.2).

use crate::message::{GrammaticalCase, GrammaticalGender, MessageValue};

/// The parameters half of §7.2's *key + parameters* abstraction: values bound to
/// names.
///
/// Arguments are **named**, not positional, because word order is exactly what
/// translation changes. A positional `{0} {1}` forces every language to keep
/// English's order or silently swap meanings; `{name} {count}` lets a translation
/// put them wherever its grammar wants.
///
/// Backed by a flat vector: a message has a handful of arguments, and a linear
/// scan over three of them beats hashing every one of them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MessageArgs {
    bindings: Vec<(Box<str>, MessageValue)>,
}

impl MessageArgs {
    /// Creates an empty argument set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds `name` to `value`, replacing any previous binding of that name.
    pub fn set(&mut self, name: impl Into<Box<str>>, value: MessageValue) {
        let name = name.into();
        if let Some(slot) = self.bindings.iter_mut().find(|(n, _)| *n == name) {
            slot.1 = value;
        } else {
            self.bindings.push((name, value));
        }
    }

    /// Binds `name` to `value` (fluent builder).
    #[must_use]
    pub fn with(mut self, name: impl Into<Box<str>>, value: MessageValue) -> Self {
        self.set(name, value);
        self
    }

    /// Binds `name` to an integer count (fluent builder).
    #[must_use]
    pub fn with_integer(self, name: impl Into<Box<str>>, count: i64) -> Self {
        self.with(name, MessageValue::integer(count))
    }

    /// Binds `name` to a literal string (fluent builder).
    #[must_use]
    pub fn with_text(self, name: impl Into<Box<str>>, text: impl Into<Box<str>>) -> Self {
        self.with(name, MessageValue::text(text))
    }

    /// Binds `name` to a grammatical gender (fluent builder; spec §7.2 gender).
    #[must_use]
    pub fn with_gender(self, name: impl Into<Box<str>>, gender: &GrammaticalGender) -> Self {
        self.with(name, MessageValue::gender(gender))
    }

    /// Binds `name` to a grammatical case (fluent builder; spec §7.2 case
    /// variants).
    #[must_use]
    pub fn with_case(self, name: impl Into<Box<str>>, case: &GrammaticalCase) -> Self {
        self.with(name, MessageValue::grammatical_case(case))
    }

    /// Returns the value bound to `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MessageValue> {
        self.bindings
            .iter()
            .find(|(n, _)| &**n == name)
            .map(|(_, v)| v)
    }

    /// Returns how many names are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binds_and_reads_by_name() {
        let args = MessageArgs::new()
            .with_integer("count", 21)
            .with_text("name", "Мария");
        assert_eq!(args.get("count").expect("bound").literal(), "21");
        assert_eq!(args.get("name").expect("bound").literal(), "Мария");
        assert!(args.get("missing").is_none());
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn rebinding_a_name_replaces_rather_than_shadows() {
        let args = MessageArgs::new().with_integer("n", 1).with_integer("n", 5);
        assert_eq!(args.len(), 1);
        assert_eq!(args.get("n").expect("bound").literal(), "5");
    }

    #[test]
    fn an_empty_set_is_empty() {
        assert!(MessageArgs::new().is_empty());
        assert!(!MessageArgs::new().with_integer("n", 0).is_empty());
    }

    #[test]
    fn gender_and_case_bind_by_name() {
        let args = MessageArgs::new()
            .with_gender("actor", &GrammaticalGender::feminine())
            .with_case("noun", &GrammaticalCase::genitive());
        assert_eq!(
            args.get("actor").expect("bound").selector_name(),
            "feminine"
        );
        assert_eq!(args.get("noun").expect("bound").selector_name(), "genitive");
    }
}
