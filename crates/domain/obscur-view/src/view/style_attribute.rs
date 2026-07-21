//! Well-known style attributes carried on element view nodes for CSS matching.
//!
//! CSS selectors match on an element's *type* (local name), its `id`, and its
//! `class` list (spec §4.13). Those live on a [`ViewNode`](super::ViewNode)
//! element as ordinary [`ViewProperty`] bindings under a small set of reserved
//! interned symbols, defined here so the whole toolkit agrees on them. The
//! reconciler reads them via [`StyleAttributes::extract`] and stores the result
//! on the retained node, where the selector matcher can see it.
//!
//! The reserved names occupy a low symbol namespace (`0x0000_00xx`), distinct
//! from the component/primitive prop namespaces (e.g. `0x00C1_xxxx`), so they
//! never collide with an ordinary prop.

use obscur_identity::InternedSymbol;

use super::virtual_collection::VIRTUAL_ROW_EXTENT;
use super::{ViewProperty, ViewValue};

/// Reserved symbol for the element's `class` attribute — a whitespace-separated
/// list of class names (as [`ViewValue::Text`]).
pub const CLASS: InternedSymbol = InternedSymbol::from_raw(0x0000_0001);

/// Reserved symbol for the element's `id` attribute (as [`ViewValue::Text`]).
pub const ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0002);

/// Reserved symbol for the element's *type* / local name — the tag a CSS type
/// selector matches (as [`ViewValue::Text`]).
pub const TYPE: InternedSymbol = InternedSymbol::from_raw(0x0000_0003);

/// Reserved symbol for a replaced element's image *source* — the opaque key an
/// `<img>` / [`Image`](../../../obscur_primitives/struct.Image.html) primitive
/// carries (as [`ViewValue::Text`]). Unlike `class` / `id` / `type`, this is not a
/// CSS selector input: it is the element's own *content*, resolved by the runtime
/// to decoded pixels and painted over the element's box. Kept in the same reserved
/// low-symbol namespace so it survives reconciliation onto the retained node.
pub const SRC: InternedSymbol = InternedSymbol::from_raw(0x0000_0004);

/// Reserved symbol for a control's `disabled` structural state — the `:disabled`
/// pseudo-class input (spec §4.13). Unlike `class`/`id`/`type` (selector inputs
/// matched as strings), this is a *boolean* state the element carries about
/// itself: present with a truthy [`ViewValue`] means disabled. A widget emits it
/// **only when true**, so toggling it off drops the attribute and the extracted
/// [`StyleAttributes`] change, driving a restyle. Kept in the same reserved
/// low-symbol namespace so it survives reconciliation onto the retained node.
pub const DISABLED: InternedSymbol = InternedSymbol::from_raw(0x0000_0005);

/// Reserved symbol for a control's `checked` structural state — the `:checked`
/// pseudo-class input (spec §4.13). Boolean state (present + truthy = checked);
/// emitted **only when true** so a toggle re-styles. Same reserved namespace as
/// `disabled`, so it survives reconciliation.
pub const CHECKED: InternedSymbol = InternedSymbol::from_raw(0x0000_0006);

/// Reserved symbol for a text control's `readonly` structural state — the HTML
/// `readonly` attribute (WHATWG HTML §attr-input-readonly). **Distinct from
/// [`DISABLED`]:** a readonly control is still *focusable* and its text is still
/// *selectable / copyable*; it merely cannot be *edited* (no insert/delete, no IME
/// commit, no paste). Boolean state (present + truthy = readonly); emitted **only
/// when true** so toggling it drops the attribute and re-styles. Same reserved
/// namespace as `disabled`/`checked`, so it survives reconciliation onto the
/// retained node, where the runtime reads it to gate *only* the mutation path.
pub const READONLY: InternedSymbol = InternedSymbol::from_raw(0x0000_0007);

/// Reserved symbol for a form control's **controlled value** — the authoritative
/// value the application declares for the field (WHATWG HTML §4.10.5.5 *value
/// reflection*; obscur.kit §3.1 "values flow down"). Like [`SRC`] (and unlike
/// `class`/`id`/`type`) this is **not** a CSS selector input: it is the field's
/// own model value, carried verbatim (as [`ViewValue::Text`]). A widget emits it
/// to declare "this field is controlled — its buffer must reflect this value";
/// the runtime reads it to snap a focused editable's buffer back when the app
/// holds the value constant while the user typed (a refused proposal). A bare
/// editable that emits no `VALUE` is its own source of truth and is never snapped.
/// Same reserved low-symbol namespace so it survives reconciliation.
pub const VALUE: InternedSymbol = InternedSymbol::from_raw(0x0000_0008);

/// Reserved symbol for a text control's **placeholder** — the hint shown while the
/// field is empty (the CSS `::placeholder` pseudo-element, shown while
/// `:placeholder-shown` holds). Like [`VALUE`] (and unlike `class`/`id`/`type`)
/// this is **not** a CSS selector input: it is the field's own placeholder string,
/// carried verbatim (as [`ViewValue::Text`]). A widget emits it so the runtime can
/// paint the placeholder as a §4.16 synthetic part — paint-only, DevTools-visible,
/// hit-transparent, and **not** a component-tree child — rather than materialising
/// it into the tree. Same reserved low-symbol namespace so it survives
/// reconciliation.
pub const PLACEHOLDER: InternedSymbol = InternedSymbol::from_raw(0x0000_0009);

/// The CSS-relevant identity of an element view node: its local name, `id`, and
/// class list, extracted from a node's [`ViewProperty`] bindings.
///
/// Names are kept as owned strings (not interned) because CSS selector names
/// are matched as strings — the authored stylesheet and the tree intern names
/// in separate tables, so string comparison is the only coherent match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleAttributes {
    local_name: Option<Box<str>>,
    id: Option<Box<str>>,
    classes: Vec<Box<str>>,
    /// A replaced element's image source key (`<img src>`), if any. Not a CSS
    /// selector input — it is the element's own content, resolved to pixels and
    /// painted by the runtime. Derived from the [`SRC`] reserved symbol.
    image_source: Option<Box<str>>,
    /// Whether the control is disabled — the `:disabled` pseudo-class input.
    /// Derived from the [`DISABLED`] reserved symbol.
    disabled: bool,
    /// Whether the control is checked — the `:checked` pseudo-class input.
    /// Derived from the [`CHECKED`] reserved symbol.
    checked: bool,
    /// Whether the text control is readonly — the HTML `readonly` attribute
    /// (WHATWG HTML §attr-input-readonly). Derived from the [`READONLY`] reserved
    /// symbol. Independent of [`disabled`](Self::disabled): a readonly control is
    /// focusable + selectable but not editable.
    readonly: bool,
    /// The control's declared **controlled value** (WHATWG HTML §4.10.5.5), if the
    /// widget emits one. Derived from the [`VALUE`] reserved symbol. Not a CSS
    /// selector input — it is the field's model value the runtime reflects into the
    /// editing buffer. Absent for a bare editable (its text is its only value).
    control_value: Option<Box<str>>,
    /// A realized virtual-list spacer's extent along the scroll axis, in logical
    /// pixels — the space the rows outside the window would have taken.
    ///
    /// Not a CSS input: it changes with the scroll position and the model, so it
    /// cannot be authored. The runtime reads it to size the spacer, which is what
    /// keeps a virtualized container's content extent (and therefore its scrollbar)
    /// true while only the window exists as nodes.
    /// Stored as whole logical pixels: the value is a span of hundreds to hundreds
    /// of thousands of them, so sub-pixel precision is meaningless, and the
    /// attribute set derives `Eq`.
    virtual_extent: Option<u32>,
    /// The text control's **placeholder** hint (the CSS `::placeholder` string),
    /// if the widget emits one. Derived from the [`PLACEHOLDER`] reserved symbol.
    /// Not a CSS selector input — the runtime reads it to paint the placeholder as
    /// a §4.16 synthetic part while the field is empty. Absent for a control with
    /// no placeholder.
    placeholder: Option<Box<str>>,
}

/// Whether a reserved boolean state attribute's value counts as *set*. Bare
/// presence ([`ViewValue::Null`]) or any truthy value is `true`; an explicit
/// `false` (`Bool(false)` / `Text("false")` / zero / empty text) is `false`.
fn is_truthy(value: &ViewValue) -> bool {
    match value {
        ViewValue::Bool(flag) => *flag,
        ViewValue::Int(n) => *n != 0,
        ViewValue::Float(f) => *f != 0.0,
        ViewValue::Text(text) => !text.is_empty() && !text.eq_ignore_ascii_case("false"),
        ViewValue::Null => true,
    }
}

impl StyleAttributes {
    /// Creates an empty attribute set (no local name, id, or classes).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Extracts the well-known style attributes from a node's properties.
    ///
    /// Reads the [`CLASS`], [`ID`], [`TYPE`], [`SRC`], [`DISABLED`], [`CHECKED`],
    /// and [`READONLY`] reserved symbols. The string-valued inputs
    /// ([`CLASS`]/[`ID`]/[`TYPE`]/[`SRC`]) contribute only from
    /// [`ViewValue::Text`] values (other value kinds are ignored); the `class`
    /// text is split on ASCII whitespace into individual class names, preserving
    /// order and dropping empties, and the [`SRC`] text is carried verbatim. The
    /// boolean state inputs ([`DISABLED`]/[`CHECKED`]/[`READONLY`]) accept any
    /// value kind and are set when it is truthy (bare presence or a non-`false`
    /// value).
    #[must_use]
    pub fn extract(props: &[ViewProperty]) -> Self {
        let mut attributes = Self::new();
        for prop in props {
            // Boolean state attributes accept any value kind (presence = set).
            match prop.name() {
                DISABLED => {
                    attributes.disabled = is_truthy(prop.value());
                    continue;
                }
                CHECKED => {
                    attributes.checked = is_truthy(prop.value());
                    continue;
                }
                READONLY => {
                    attributes.readonly = is_truthy(prop.value());
                    continue;
                }
                _ => {}
            }
            // The spacer extent is numeric, so it is read before the string block.
            if prop.name() == VIRTUAL_ROW_EXTENT {
                attributes.virtual_extent = match prop.value() {
                    ViewValue::Float(px) => (*px >= 0.0).then(|| px.round() as u32),
                    ViewValue::Int(px) => u32::try_from(*px).ok(),
                    _ => None,
                };
                continue;
            }
            // The remaining reserved attributes are string-valued selector inputs.
            let ViewValue::Text(text) = prop.value() else {
                continue;
            };
            match prop.name() {
                CLASS => {
                    attributes.classes = text.split_ascii_whitespace().map(Into::into).collect();
                }
                ID => attributes.id = Some(text.clone()),
                TYPE => attributes.local_name = Some(text.clone()),
                SRC => attributes.image_source = Some(text.clone()),
                VALUE => attributes.control_value = Some(text.clone()),
                PLACEHOLDER => attributes.placeholder = Some(text.clone()),
                _ => {}
            }
        }
        attributes
    }

    /// The element's local name (type), if set.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_deref()
    }

    /// The element's `id`, if set.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The element's class list (possibly empty).
    #[must_use]
    pub fn classes(&self) -> &[Box<str>] {
        &self.classes
    }

    /// Whether this element carries a given class.
    #[must_use]
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c.as_ref() == class)
    }

    /// The element's image source key (`<img src>`), if any. This is the
    /// element's own content — the runtime resolves it to decoded pixels and
    /// paints it over the element's box.
    #[must_use]
    pub fn image_source(&self) -> Option<&str> {
        self.image_source.as_deref()
    }

    /// Whether the control reports the `disabled` structural state (`:disabled`).
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Whether the control reports the `checked` structural state (`:checked`).
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        self.checked
    }

    /// Whether the text control reports the `readonly` state (HTML `readonly`).
    /// Independent of [`is_disabled`](Self::is_disabled): a readonly control is
    /// focusable + selectable but not editable.
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.readonly
    }

    /// The control's declared controlled value (WHATWG HTML §4.10.5.5), if the
    /// widget emitted one. `None` for a bare editable whose text is its own value.
    #[must_use]
    pub fn control_value(&self) -> Option<&str> {
        self.control_value.as_deref()
    }

    /// The text control's placeholder hint (the CSS `::placeholder` string), if
    /// the widget emitted one. `None` for a control with no placeholder.
    #[must_use]
    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    /// A realized virtual-list spacer's extent, if this node is one.
    #[must_use]
    pub fn virtual_extent(&self) -> Option<f32> {
        self.virtual_extent.map(|px| px as f32)
    }

    /// Whether no style attribute is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.local_name.is_none()
            && self.id.is_none()
            && self.classes.is_empty()
            && self.image_source.is_none()
            && !self.disabled
            && !self.checked
            && !self.readonly
            && self.control_value.is_none()
            && self.placeholder.is_none()
            && self.virtual_extent.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_prop(name: InternedSymbol, value: &str) -> ViewProperty {
        ViewProperty::new(name, ViewValue::Text(value.into()))
    }

    #[test]
    fn empty_when_no_attributes() {
        let attrs = StyleAttributes::extract(&[]);
        assert!(attrs.is_empty());
        assert_eq!(attrs.local_name(), None);
        assert_eq!(attrs.id(), None);
        assert!(attrs.classes().is_empty());
    }

    #[test]
    fn extracts_local_name_id_and_classes() {
        let props = [
            text_prop(TYPE, "container"),
            text_prop(ID, "main"),
            text_prop(CLASS, "counter  primary   large"),
        ];
        let attrs = StyleAttributes::extract(&props);
        assert_eq!(attrs.local_name(), Some("container"));
        assert_eq!(attrs.id(), Some("main"));
        let classes: Vec<&str> = attrs.classes().iter().map(AsRef::as_ref).collect();
        assert_eq!(classes, ["counter", "primary", "large"]);
        assert!(attrs.has_class("primary"));
        assert!(!attrs.has_class("missing"));
        assert!(!attrs.is_empty());
    }

    #[test]
    fn extracts_image_source_from_src() {
        let attrs = StyleAttributes::extract(&[text_prop(SRC, "logo.png")]);
        assert_eq!(attrs.image_source(), Some("logo.png"));
        // The image source alone makes the attribute set non-empty.
        assert!(!attrs.is_empty());
        // It is independent of the CSS selector inputs.
        assert_eq!(attrs.local_name(), None);
        assert_eq!(attrs.id(), None);
        assert!(attrs.classes().is_empty());
    }

    #[test]
    fn equality_includes_image_source() {
        let a = StyleAttributes::extract(&[text_prop(SRC, "a.png")]);
        let b = StyleAttributes::extract(&[text_prop(SRC, "b.png")]);
        // A src change is observed by equality (so reconcile re-applies attributes).
        assert_ne!(a, b);
        let a2 = StyleAttributes::extract(&[text_prop(SRC, "a.png")]);
        assert_eq!(a, a2);
    }

    #[test]
    fn extracts_disabled_and_checked_state_flags() {
        // Boolean state attributes accept any value kind; a truthy value sets them.
        let attrs = StyleAttributes::extract(&[
            ViewProperty::new(DISABLED, ViewValue::Bool(true)),
            ViewProperty::new(CHECKED, ViewValue::Bool(true)),
        ]);
        assert!(attrs.is_disabled());
        assert!(attrs.is_checked());
        // A state flag alone makes the set non-empty (it is a restyle input).
        assert!(!attrs.is_empty());

        // Bare presence (Null) also counts as set.
        let present = StyleAttributes::extract(&[
            ViewProperty::new(DISABLED, ViewValue::Null),
            text_prop(CHECKED, "true"),
        ]);
        assert!(present.is_disabled());
        assert!(present.is_checked());

        // An explicit false value does not set the flag.
        let off = StyleAttributes::extract(&[
            ViewProperty::new(DISABLED, ViewValue::Bool(false)),
            text_prop(CHECKED, "false"),
        ]);
        assert!(!off.is_disabled());
        assert!(!off.is_checked());
        assert!(off.is_empty());
    }

    #[test]
    fn equality_includes_state_flags() {
        // A state change is observed by equality, so reconcile re-applies attributes
        // and the node restyles.
        let disabled =
            StyleAttributes::extract(&[ViewProperty::new(DISABLED, ViewValue::Bool(true))]);
        let enabled = StyleAttributes::extract(&[]);
        assert_ne!(disabled, enabled);

        let checked =
            StyleAttributes::extract(&[ViewProperty::new(CHECKED, ViewValue::Bool(true))]);
        let unchecked = StyleAttributes::extract(&[]);
        assert_ne!(checked, unchecked);
    }

    #[test]
    fn extracts_readonly_state_flag_independent_of_disabled() {
        // `readonly` is a distinct boolean state from `disabled` (WHATWG HTML
        // §attr-input-readonly): a field can carry either, both, or neither.
        let readonly =
            StyleAttributes::extract(&[ViewProperty::new(READONLY, ViewValue::Bool(true))]);
        assert!(readonly.is_readonly());
        assert!(!readonly.is_disabled());
        assert!(!readonly.is_empty());

        // Bare presence counts as set; an explicit false does not.
        assert!(
            StyleAttributes::extract(&[ViewProperty::new(READONLY, ViewValue::Null)]).is_readonly()
        );
        let off = StyleAttributes::extract(&[ViewProperty::new(READONLY, ViewValue::Bool(false))]);
        assert!(!off.is_readonly());
        assert!(off.is_empty());

        // Both flags can be set at once.
        let both = StyleAttributes::extract(&[
            ViewProperty::new(DISABLED, ViewValue::Bool(true)),
            ViewProperty::new(READONLY, ViewValue::Bool(true)),
        ]);
        assert!(both.is_disabled() && both.is_readonly());
    }

    #[test]
    fn equality_includes_readonly_flag() {
        // A readonly change is observed by equality, so reconcile re-applies
        // attributes and the runtime sees the state change.
        let readonly =
            StyleAttributes::extract(&[ViewProperty::new(READONLY, ViewValue::Bool(true))]);
        let normal = StyleAttributes::extract(&[]);
        assert_ne!(readonly, normal);
    }

    #[test]
    fn ignores_non_text_and_unknown_symbols() {
        let props = [
            ViewProperty::new(CLASS, ViewValue::Int(7)),
            ViewProperty::new(
                InternedSymbol::from_raw(0x00C1_0501),
                ViewValue::Text("w".into()),
            ),
        ];
        let attrs = StyleAttributes::extract(&props);
        assert!(attrs.is_empty());
    }

    #[test]
    fn blank_class_yields_no_classes() {
        let attrs = StyleAttributes::extract(&[text_prop(CLASS, "   ")]);
        assert!(attrs.classes().is_empty());
    }
}
