#![doc = include_str!("message_args.doc/_module.md")]

use crate::message::{GrammaticalCase, GrammaticalGender, MessageValue};

#[doc = include_str!("message_args.doc/MessageArgs.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MessageArgs {
    bindings: Vec<(Box<str>, MessageValue)>,
}

impl MessageArgs {
    #[doc = include_str!("message_args.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("message_args.doc/set.md")]
    pub fn set(&mut self, name: impl Into<Box<str>>, value: MessageValue) {
        let name = name.into();
        if let Some(slot) = self.bindings.iter_mut().find(|(n, _)| *n == name) {
            slot.1 = value;
        } else {
            self.bindings.push((name, value));
        }
    }

    #[doc = include_str!("message_args.doc/with.md")]
    #[must_use]
    pub fn with(mut self, name: impl Into<Box<str>>, value: MessageValue) -> Self {
        self.set(name, value);
        self
    }

    #[doc = include_str!("message_args.doc/with_integer.md")]
    #[must_use]
    pub fn with_integer(self, name: impl Into<Box<str>>, count: i64) -> Self {
        self.with(name, MessageValue::integer(count))
    }

    #[doc = include_str!("message_args.doc/with_text.md")]
    #[must_use]
    pub fn with_text(self, name: impl Into<Box<str>>, text: impl Into<Box<str>>) -> Self {
        self.with(name, MessageValue::text(text))
    }

    #[doc = include_str!("message_args.doc/with_gender.md")]
    #[must_use]
    pub fn with_gender(self, name: impl Into<Box<str>>, gender: &GrammaticalGender) -> Self {
        self.with(name, MessageValue::gender(gender))
    }

    #[doc = include_str!("message_args.doc/with_case.md")]
    #[must_use]
    pub fn with_case(self, name: impl Into<Box<str>>, case: &GrammaticalCase) -> Self {
        self.with(name, MessageValue::grammatical_case(case))
    }

    #[doc = include_str!("message_args.doc/get.md")]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MessageValue> {
        self.bindings
            .iter()
            .find(|(n, _)| &**n == name)
            .map(|(_, v)| v)
    }

    #[doc = include_str!("message_args.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    #[doc = include_str!("message_args.doc/is_empty.md")]
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
