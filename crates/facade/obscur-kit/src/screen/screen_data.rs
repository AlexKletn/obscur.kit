//! [`ScreenData`] — the named values a `.okui` screen's bindings read each frame.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::view::view::ViewValue;

/// The values an [`OkuiScreen`](super::OkuiScreen)'s `{bindings}` resolve against,
/// shared by handle (spec §2.8, §2.9).
///
/// # Why this exists
///
/// A `.okui` file is markup, and the two ways to turn it into a view differ in
/// *where the bindings are resolved*:
///
/// - the **build-time** route (`build.rs` →
///   [`ViewCodegen`](crate::okui_compiler::codegen::ViewCodegen)) emits ordinary
///   Rust, so `{count}` is compiled to a read of the application's own `count`
///   binding — full reactivity, and §2.13's contract validation happens while the
///   crate is built;
/// - the **run-time** route (`App::screen` /
///   [`OkuiScreen`](super::OkuiScreen)) parses and compiles the source inside
///   `run`, where no Rust locals exist to read. The IR keeps a binding as a
///   *path*, and the runtime bridge — which cannot evaluate anything — lowers that
///   path to placeholder text.
///
/// This table is what that second route resolves paths against: an application
/// binds `"count"` here, the screen reads it while it lowers, and a value written
/// between two frames is visible on the second. Before it, a `.screen` was
/// materialised **once** and no state change could reach it at all; the builder
/// returned the same pre-built `View` on every frame.
///
/// # Semantics
///
/// A handle is cheap to [`clone`](Clone) and every clone names the same table, so
/// a command handler can capture one and write to it (a handler receives no
/// runtime and must not touch one — see
/// [`AppControl`](crate::control::AppControl) — but this is plain application
/// state, so writing it needs no deferral). A name with no value bound is not an
/// error: the screen keeps the bridge's placeholder, so an unbound `{count}`
/// renders the *path* and is visible rather than silently blank.
///
/// Values are [`ViewValue`]s because that is what a property lowers to; text
/// content renders a value through [`ScreenData::render`], which is how an `Int`
/// bound to `{count}` becomes the characters `7`.
#[derive(Clone, Default)]
pub struct ScreenData {
    values: Rc<RefCell<HashMap<Box<str>, ViewValue>>>,
}

impl ScreenData {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds `name` to `value`, replacing any previous value.
    ///
    /// Takes `&self`: every clone of the handle names one table, so a handler that
    /// captured a clone writes what the next frame reads. Writing does **not**
    /// schedule a frame — the runtime schedules one for the input that ran the
    /// handler, and a write from outside a handler pairs with
    /// [`AppControl::request_frame`](crate::control::AppControl::request_frame).
    pub fn set(&self, name: &str, value: impl Into<ViewValue>) {
        self.values.borrow_mut().insert(name.into(), value.into());
    }

    /// The value bound to `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ViewValue> {
        self.values.borrow().get(name).cloned()
    }

    /// The value bound to `name` as an integer, or `0` when it is unbound or not
    /// an [`Int`](ViewValue::Int).
    ///
    /// The read half of the read-modify-write a counter needs
    /// (`data.set("count", data.int("count") + 1)`), which is otherwise three
    /// lines of matching in every handler.
    #[must_use]
    pub fn int(&self, name: &str) -> i64 {
        match self.values.borrow().get(name) {
            Some(ViewValue::Int(value)) => *value,
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
        self.values.borrow_mut().remove(name).is_some()
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

    /// Every bound name, sorted.
    ///
    /// This is what [`OkuiScreen::compile_with`](super::OkuiScreen::compile_with)
    /// **declares to the compiler**, so a screen may reference these names and
    /// referencing anything else is a §2.13 compile error rather than a silently
    /// missing value. Sorted so the declaration — and therefore any diagnostic order
    /// — does not depend on hash iteration order.
    #[must_use]
    pub fn names(&self) -> Vec<Box<str>> {
        let mut names: Vec<Box<str>> = self.values.borrow().keys().cloned().collect();
        names.sort_unstable();
        names
    }

    /// How `value` reads as **text content** — what a body `{binding}` renders.
    ///
    /// A [`Text`](ViewValue::Text) is itself; a number or a boolean is its ordinary
    /// `Display` form (`7`, `1.5`, `true`), which is the same rule the build-time
    /// route follows, where the emitted Rust interpolates the value with
    /// `Display`. [`Null`](ViewValue::Null) renders empty rather than the literal
    /// word "null" — a bound-but-absent value is *nothing to show*, not a noun.
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
