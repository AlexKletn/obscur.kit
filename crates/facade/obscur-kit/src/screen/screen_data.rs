#![doc = include_str!("screen_data.doc/_module.md")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::view::view::ViewValue;

#[doc = include_str!("screen_data.doc/ScreenData.md")]
#[derive(Clone, Default)]
pub struct ScreenData {
    values: Rc<RefCell<HashMap<Box<str>, ViewValue>>>,
}

impl ScreenData {
    #[doc = include_str!("screen_data.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("screen_data.doc/set.md")]
    pub fn set(&self, name: &str, value: impl Into<ViewValue>) {
        self.values.borrow_mut().insert(name.into(), value.into());
    }

    #[doc = include_str!("screen_data.doc/get.md")]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ViewValue> {
        self.values.borrow().get(name).cloned()
    }

    #[doc = include_str!("screen_data.doc/int.md")]
    #[must_use]
    pub fn int(&self, name: &str) -> i64 {
        match self.values.borrow().get(name) {
            Some(ViewValue::Int(value)) => *value,
            _ => 0,
        }
    }

    #[doc = include_str!("screen_data.doc/is_bound.md")]
    #[must_use]
    pub fn is_bound(&self, name: &str) -> bool {
        self.values.borrow().contains_key(name)
    }

    #[doc = include_str!("screen_data.doc/remove.md")]
    #[must_use = "the return says whether the name was bound; drop it with `let _ =`"]
    pub fn remove(&self, name: &str) -> bool {
        self.values.borrow_mut().remove(name).is_some()
    }

    #[doc = include_str!("screen_data.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    #[doc = include_str!("screen_data.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    #[doc = include_str!("screen_data.doc/names.md")]
    #[must_use]
    pub fn names(&self) -> Vec<Box<str>> {
        let mut names: Vec<Box<str>> = self.values.borrow().keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[doc = include_str!("screen_data.doc/render.md")]
    #[must_use]
    pub fn render(value: &ViewValue) -> Box<str> {
        match value {
            ViewValue::Null => Box::from(""),
            ViewValue::Bool(flag) => Box::from(if *flag { "true" } else { "false" }),
            ViewValue::Int(number) => number.to_string().into_boxed_str(),
            ViewValue::Float(number) => number.to_string().into_boxed_str(),
            ViewValue::Text(text) => text.clone(),
        }
    }
}

impl std::fmt::Debug for ScreenData {
    #[doc = include_str!("screen_data.doc/fmt.md")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self.values.borrow();
        let mut names: Vec<&str> = values.keys().map(AsRef::as_ref).collect();
        names.sort_unstable();
        f.debug_struct("ScreenData").field("bound", &names).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{ScreenData, ViewValue};

    #[test]
    fn every_clone_of_a_handle_names_one_table() {
        // The property a handler depends on: it captures a clone, and the screen
        // reads the original.
        let data = ScreenData::new();
        let handle = data.clone();
        handle.set("count", 3_i64);
        assert_eq!(data.get("count"), Some(ViewValue::Int(3)));
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn a_write_replaces_rather_than_accumulates() {
        let data = ScreenData::new();
        data.set("count", 1_i64);
        data.set("count", 2_i64);
        assert_eq!(data.int("count"), 2);
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn an_unbound_or_mistyped_integer_reads_as_zero() {
        let data = ScreenData::new();
        assert_eq!(data.int("count"), 0, "unbound");
        data.set("count", "seven");
        assert_eq!(data.int("count"), 0, "bound, but not an integer");
    }

    #[test]
    fn removing_a_name_unbinds_it() {
        let data = ScreenData::new();
        data.set("count", 1_i64);
        assert!(data.remove("count"));
        assert!(!data.remove("count"));
        assert!(!data.is_bound("count"));
        assert!(data.is_empty());
    }

    #[test]
    fn text_content_renders_a_value_the_way_the_build_time_route_would() {
        // `Display`, not `Debug`: `{count}` must render `7`, not `Int(7)`.
        assert_eq!(&*ScreenData::render(&ViewValue::Int(7)), "7");
        assert_eq!(&*ScreenData::render(&ViewValue::Bool(true)), "true");
        assert_eq!(&*ScreenData::render(&ViewValue::Text("ada".into())), "ada");
        assert_eq!(
            &*ScreenData::render(&ViewValue::Null),
            "",
            "a bound-but-absent value shows nothing, not the word null"
        );
    }
}
