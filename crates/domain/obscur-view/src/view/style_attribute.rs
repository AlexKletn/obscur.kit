#![doc = include_str!("../../docs/api/view/style_attribute.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::virtual_collection::{VIRTUAL_LIST_ID, VIRTUAL_ROW_EXTENT};
use super::{ViewProperty, ViewValue};

/// Reserved symbol for the element's `class` attribute — a whitespace-separated
/// list of class names (as [`ViewValue::Text`]).
pub const CLASS: InternedSymbol = InternedSymbol::from_raw(0x0000_0001);

/// Reserved symbol for the element's `id` attribute (as [`ViewValue::Text`]).
pub const ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0002);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/TYPE.md")]
pub const TYPE: InternedSymbol = InternedSymbol::from_raw(0x0000_0003);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/SRC.md")]
pub const SRC: InternedSymbol = InternedSymbol::from_raw(0x0000_0004);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/DISABLED.md")]
pub const DISABLED: InternedSymbol = InternedSymbol::from_raw(0x0000_0005);

/// Reserved symbol for a control's `checked` structural state — the `:checked`
/// pseudo-class input (spec §4.13). Boolean state (present + truthy = checked);
/// emitted **only when true** so a toggle re-styles. Same reserved namespace as
/// `disabled`, so it survives reconciliation.
pub const CHECKED: InternedSymbol = InternedSymbol::from_raw(0x0000_0006);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/READONLY.md")]
pub const READONLY: InternedSymbol = InternedSymbol::from_raw(0x0000_0007);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/VALUE.md")]
pub const VALUE: InternedSymbol = InternedSymbol::from_raw(0x0000_0008);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/PLACEHOLDER.md")]
pub const PLACEHOLDER: InternedSymbol = InternedSymbol::from_raw(0x0000_0009);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/LOCAL_NAME.md")]
pub const LOCAL_NAME: InternedSymbol = InternedSymbol::from_raw(0x0000_000A);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/STYLE.md")]
pub const STYLE: InternedSymbol = InternedSymbol::from_raw(0x0000_000B);

#[doc = include_str!("../../docs/api/view/style_attribute.doc/StyleAttributes.md")]
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
    #[doc = include_str!("../../docs/api/view/style_attribute.doc/virtual_extent.md")]
    virtual_extent: Option<u32>,
    /// The virtual list a realized spacer belongs to, so the runtime can find the
    /// container it sits in and use that container's scroll offset as the window's.
    virtual_list: Option<u32>,
    /// The text control's **placeholder** hint (the CSS `::placeholder` string),
    /// if the widget emits one. Derived from the [`PLACEHOLDER`] reserved symbol.
    /// Not a CSS selector input — the runtime reads it to paint the placeholder as
    /// a §4.16 synthetic part while the field is empty. Absent for a control with
    /// no placeholder.
    placeholder: Option<Box<str>>,
    /// The authored `type="…"` attribute ([`TYPE`]) — HTML's "which kind", kept apart
    /// from the element's own [`local_name`](StyleAttributes::local_name).
    control_type: Option<Box<str>>,
    /// The declarations written on the node itself, uncompiled.
    inline_style: Option<Box<str>>,
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

fn virtual_extent_px(value: &ViewValue) -> Option<u32> {
    match value {
        ViewValue::Int(px) => u32::try_from(*px).ok(),
        ViewValue::Float(px) if *px >= 0.0 => {
            let rounded_px = px.round();
            (rounded_px <= f64::from(u32::MAX)).then(|| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "the rounded value is within the nonnegative u32 range"
                )]
                let extent_px = rounded_px as u32;
                extent_px
            })
        }
        _ => None,
    }
}

impl StyleAttributes {
    /// Creates an empty attribute set (no local name, id, or classes).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/view/style_attribute.doc/extract.md")]
    #[must_use]
    pub fn extract(props: &[ViewProperty]) -> Self {
        let mut attributes = Self::new();
        for prop in props {
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
            if prop.name() == VIRTUAL_ROW_EXTENT {
                attributes.virtual_extent = virtual_extent_px(prop.value());
                continue;
            }
            if prop.name() == VIRTUAL_LIST_ID {
                attributes.virtual_list = match prop.value() {
                    ViewValue::Int(raw) => u32::try_from(*raw).ok(),
                    _ => None,
                };
                continue;
            }
            let ViewValue::Text(text) = prop.value() else {
                continue;
            };
            match prop.name() {
                CLASS => {
                    attributes.classes = text.split_ascii_whitespace().map(Into::into).collect();
                }
                ID => attributes.id = Some(text.clone()),
                LOCAL_NAME => attributes.local_name = Some(text.clone()),
                TYPE => attributes.control_type = Some(text.clone()),
                SRC => attributes.image_source = Some(text.clone()),
                VALUE => attributes.control_value = Some(text.clone()),
                PLACEHOLDER => attributes.placeholder = Some(text.clone()),
                STYLE => attributes.inline_style = Some(text.clone()),
                _ => {}
            }
        }
        attributes
    }

    /// The `style` attribute's text, if the node carries one.
    #[must_use]
    pub fn inline_style(&self) -> Option<&str> {
        self.inline_style.as_deref()
    }

    /// The element's local name (type), if set.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_deref()
    }

    #[doc = include_str!("../../docs/api/view/style_attribute.doc/control_type_2.md")]
    #[must_use]
    pub fn control_type(&self) -> Option<&str> {
        self.control_type.as_deref()
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

    /// The virtual list a spacer belongs to, if this node is one.
    #[must_use]
    pub const fn virtual_list(&self) -> Option<u32> {
        self.virtual_list
    }

    /// Whether no style attribute is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.local_name.is_none()
            && self.control_type.is_none()
            && self.id.is_none()
            && self.classes.is_empty()
            && self.image_source.is_none()
            && !self.disabled
            && !self.checked
            && !self.readonly
            && self.control_value.is_none()
            && self.placeholder.is_none()
            && self.virtual_extent.is_none()
            && self.virtual_list.is_none()
            && self.inline_style.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_prop(name: InternedSymbol, value: &str) -> ViewProperty {
        ViewProperty::new(name, ViewValue::Text(value.into()))
    }

    #[test]
    fn every_reserved_style_attribute_symbol_is_distinct() {
        // The dense low-range table is hand-numbered, so a duplicated literal is a
        // silent aliasing bug: two attributes would write one field. `LOCAL_NAME`
        // was the newest entry and the one this most needed to catch — it was
        // carved out of `TYPE`, and a value equal to it would have re-created the
        // exact collision it exists to end.
        let reserved = [
            ("CLASS", CLASS),
            ("ID", ID),
            ("TYPE", TYPE),
            ("SRC", SRC),
            ("DISABLED", DISABLED),
            ("CHECKED", CHECKED),
            ("READONLY", READONLY),
            ("VALUE", VALUE),
            ("PLACEHOLDER", PLACEHOLDER),
            ("LOCAL_NAME", LOCAL_NAME),
            ("STYLE", STYLE),
            ("VIRTUAL_ROW_EXTENT", VIRTUAL_ROW_EXTENT),
            ("VIRTUAL_LIST_ID", VIRTUAL_LIST_ID),
        ];
        for (i, (left_name, left)) in reserved.iter().enumerate() {
            for (right_name, right) in &reserved[i + 1..] {
                assert_ne!(left, right, "{left_name} aliases {right_name}");
            }
        }
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
    fn virtual_extent_accepts_only_representable_nonnegative_pixels() {
        assert_eq!(virtual_extent_px(&ViewValue::Float(12.5)), Some(13));
        assert_eq!(
            virtual_extent_px(&ViewValue::Float(f64::from(u32::MAX))),
            Some(u32::MAX)
        );
        for px in [-0.1, f64::INFINITY, f64::NAN, f64::from(u32::MAX) + 0.5] {
            let props = [ViewProperty::new(VIRTUAL_ROW_EXTENT, ViewValue::Float(px))];
            assert_eq!(StyleAttributes::extract(&props).virtual_extent(), None);
        }
    }

    #[test]
    fn a_node_carrying_only_a_style_attribute_is_not_empty() {
        // `is_empty` is what the tree uses to decide a node has nothing the style
        // tier needs to look at. Omitting `inline_style` from it would make an
        // element whose only attribute is `style="…"` indistinguishable from a
        // bare one — the declarations would be extracted and then never asked for.
        let attrs = StyleAttributes::extract(&[text_prop(STYLE, "opacity:0.5")]);
        assert!(!attrs.is_empty());
        assert_eq!(attrs.inline_style(), Some("opacity:0.5"));
    }

    #[test]
    fn extracts_local_name_id_and_classes() {
        let props = [
            text_prop(LOCAL_NAME, "container"),
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
    fn the_local_name_and_the_authored_type_are_two_independent_channels() {
        // The defect the LOCAL_NAME symbol exists to close: `<button
        // type="submit">` used to be an element whose CSS *type* was `submit`,
        // because one symbol carried the tag and the attribute both. The tag says
        // what a thing is; `type` says which kind; they no longer overwrite each
        // other.
        let attrs =
            StyleAttributes::extract(&[text_prop(LOCAL_NAME, "button"), text_prop(TYPE, "submit")]);
        assert_eq!(attrs.local_name(), Some("button"));
        assert_eq!(attrs.control_type(), Some("submit"));

        // Either one alone leaves the other unset, and either one alone makes the
        // set non-empty (both are restyle inputs).
        let named = StyleAttributes::extract(&[text_prop(LOCAL_NAME, "button")]);
        assert_eq!(named.local_name(), Some("button"));
        assert_eq!(named.control_type(), None);
        assert!(!named.is_empty());

        let typed = StyleAttributes::extract(&[text_prop(TYPE, "submit")]);
        assert_eq!(typed.local_name(), None);
        assert_eq!(typed.control_type(), Some("submit"));
        assert!(!typed.is_empty());
    }

    #[test]
    fn equality_includes_the_authored_type() {
        // A `type` change is observed by equality, so reconcile re-applies
        // attributes and an `[type=…]` rule re-matches.
        let submit = StyleAttributes::extract(&[text_prop(TYPE, "submit")]);
        let reset = StyleAttributes::extract(&[text_prop(TYPE, "reset")]);
        assert_ne!(submit, reset);
        assert_eq!(
            submit,
            StyleAttributes::extract(&[text_prop(TYPE, "submit")])
        );
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
