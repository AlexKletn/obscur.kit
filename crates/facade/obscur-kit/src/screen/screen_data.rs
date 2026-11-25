//! [`ScreenData`] — the named values a `.okui` screen's bindings read each frame.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::view::view::ViewValue;

use super::{ScreenRow, screen_binding::ScreenBinding};

type Bindings = HashMap<Box<str>, ScreenBinding>;

#[doc = include_str!("../../docs/api/screen/screen_data.doc/ScreenData.md")]
#[derive(Clone, Default)]
pub struct ScreenData {
    // Cloned handles share the live table on the UI thread. A frame keeps its
    // own immutable snapshot, so writes copy the table only while it is shared.
    values: Rc<RefCell<Arc<Bindings>>>,
}

impl ScreenData {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/screen/screen_data.doc/set.md")]
    pub fn set(&self, name: &str, value: impl Into<ViewValue>) {
        let binding = ScreenBinding::Value(value.into());
        Arc::make_mut(&mut self.values.borrow_mut()).insert(name.into(), binding);
    }

    /// Binds a collection of records for runtime-loaded `For` and `VirtualFor`.
    /// A write replaces any previous value under the same name. Each view holds a
    /// snapshot of the records.
    pub fn set_rows(&self, name: &str, rows: impl Into<Arc<[ScreenRow]>>) {
        let binding = ScreenBinding::Rows(rows.into());
        Arc::make_mut(&mut self.values.borrow_mut()).insert(name.into(), binding);
    }

    pub(super) fn snapshot(&self) -> Arc<Bindings> {
        Arc::clone(&self.values.borrow())
    }

    /// The value bound to `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ViewValue> {
        self.values
            .borrow()
            .get(name)
            .and_then(ScreenBinding::value)
    }

    #[doc = include_str!("../../docs/api/screen/screen_data.doc/int.md")]
    #[must_use]
    pub fn int(&self, name: &str) -> i64 {
        match self.values.borrow().get(name) {
            Some(ScreenBinding::Value(ViewValue::Int(value))) => *value,
            _ => 0,
        }
    }

    /// Whether `name` has a value bound.
    #[must_use]
    pub fn is_bound(&self, name: &str) -> bool {
        self.values.borrow().contains_key(name)
    }

    /// Unbinds `name`, returning whether it was bound. The screen falls back to
    /// the binding's own path afterwards.
    #[must_use = "the return says whether the name was bound; drop it with `let _ =`"]
    pub fn remove(&self, name: &str) -> bool {
        let mut values = self.values.borrow_mut();
        values.contains_key(name) && Arc::make_mut(&mut values).remove(name).is_some()
    }

    /// How many names are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    #[doc = include_str!("../../docs/api/screen/screen_data.doc/names.md")]
    #[must_use]
    pub fn names(&self) -> Vec<Box<str>> {
        let mut names: Vec<Box<str>> = self.values.borrow().keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[doc = include_str!("../../docs/api/screen/screen_data.doc/render.md")]
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
    /// Names the bound paths without their values, so a debug print of a screen
    /// stays readable and never dumps application data.
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
    fn converting_a_value_can_read_an_existing_binding() {
        struct CopyCount<'a>(&'a ScreenData);

        impl From<CopyCount<'_>> for ViewValue {
            fn from(source: CopyCount<'_>) -> Self {
                Self::Int(source.0.int("count"))
            }
        }

        let data = ScreenData::new();
        data.set("count", 7_i64);
        data.set("copied", CopyCount(&data));
        assert_eq!(data.get("copied"), Some(ViewValue::Int(7)));
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
