use obscur_okui_compiler::validation::{
    AttributeUse, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor,
};
use proc_macro2::Span;

use super::{
    dsl_attr::DslAttr, dsl_element::DslElement, dsl_node::DslNode, is_conditional_tag, is_loop_tag,
    prop_value::PropValue,
};

#[doc = include_str!("../../../docs/api/dsl/dsl_macro.doc/DslInstance.md")]
pub(super) struct DslInstance<'a> {
    tag: &'a str,
    attributes: Vec<(&'a str, bool)>,
    attribute_spans: Vec<Span>,
    members: Vec<(&'a str, Option<SlotUse<'a>>, MemberArity)>,
    member_spans: Vec<Span>,
    /// Whether a text or `{expr}` child reaches this instance — content, which is
    /// not a member because it carries no tag.
    content: bool,
}

impl<'a> DslInstance<'a> {
    /// Describes `element` for the rule.
    pub(super) fn of(element: &'a DslElement) -> Self {
        let mut instance = Self {
            tag: element.tag.as_str(),
            attributes: Vec::new(),
            attribute_spans: Vec::new(),
            members: Vec::new(),
            member_spans: Vec::new(),
            content: false,
        };
        for attr in &element.attrs {
            let (name, event, span) = match attr {
                DslAttr::Prop { name, span, .. } => (name.as_str(), false, *span),
                DslAttr::Event { event, span, .. } => (event.as_str(), true, *span),
            };
            instance.attributes.push((name, event));
            instance.attribute_spans.push(span);
        }
        instance.collect_members(element, MemberArity::Once);
        instance
    }

    #[doc = include_str!("../../../docs/api/dsl/dsl_macro.doc/collect_members.md")]
    fn collect_members(&mut self, element: &'a DslElement, arity: MemberArity) {
        for child in &element.children {
            let DslNode::Element(child) = child else {
                self.content = true;
                continue;
            };
            if is_conditional_tag(&child.tag) {
                self.collect_members(child, degrade(arity, MemberArity::AtMostOnce));
                continue;
            }
            if is_loop_tag(&child.tag) {
                self.collect_members(child, MemberArity::Any);
                continue;
            }
            // A `<For>`'s `<Empty>`/`<Separator>` are §2.11 slots whose bodies
            // reach the loop's parent; the wrappers themselves are never nodes.
            if arity == MemberArity::Any
                && matches!(child.tag.as_str(), "Empty" | "Loading" | "Separator")
            {
                self.collect_members(child, MemberArity::Any);
                continue;
            }
            self.members
                .push((child.tag.as_str(), slot_of(child), arity));
            self.member_spans.push(child.span);
        }
    }

    /// Borrows this instance as the plain [`ElementUse`] the rule takes.
    pub(super) fn as_use(&self) -> ElementUse<'_> {
        ElementUse {
            tag: self.tag,
            attributes: self
                .attributes
                .iter()
                .map(|(name, event)| AttributeUse {
                    name,
                    event: *event,
                })
                .collect(),
            members: self
                .members
                .iter()
                .map(|(tag, slot, arity)| MemberUse {
                    tag,
                    slot: *slot,
                    arity: *arity,
                })
                .collect(),
            content: self.content,
        }
    }

    /// Resolves an anchor to the span its error is reported at.
    pub(super) fn span_of(&self, anchor: ViolationAnchor, element: Span) -> Span {
        match anchor {
            ViolationAnchor::Element => element,
            ViolationAnchor::Attribute(index) => {
                self.attribute_spans.get(index).copied().unwrap_or(element)
            }
            ViolationAnchor::Member(index) => {
                self.member_spans.get(index).copied().unwrap_or(element)
            }
        }
    }
}

#[doc = include_str!("../../../docs/api/dsl/dsl_macro.doc/slot_of.md")]
fn slot_of(element: &DslElement) -> Option<SlotUse<'_>> {
    element.attrs.iter().find_map(|attr| match attr {
        DslAttr::Prop { name, value, .. } if name == "slot" => Some(match value {
            PropValue::Text(text) => SlotUse::Named(text.as_str()),
            _ => SlotUse::Dynamic,
        }),
        _ => None,
    })
}

/// Weakens `arity` to `inner`, keeping the weaker of the two: nesting only ever
/// loosens the bound.
const fn degrade(arity: MemberArity, inner: MemberArity) -> MemberArity {
    match arity {
        MemberArity::Any => MemberArity::Any,
        MemberArity::Once | MemberArity::AtMostOnce => inner,
    }
}
