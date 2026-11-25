use obscur_okui_syntax::ast::Element;
use obscur_okui_syntax::syntax::SyntaxKind;

use crate::lowering::event_local_name;
use crate::validation::contract_rule::{
    AttributeUse, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor,
};

use super::slot::Slot;

#[doc = include_str!("../../../docs/api/validation/contract_validator.doc/AstInstance.md")]
pub(super) struct AstInstance {
    tag: String,
    attributes: Vec<(String, bool)>,
    attribute_offsets: Vec<u32>,
    members: Vec<(String, Option<Slot>, MemberArity)>,
    member_offsets: Vec<u32>,
    /// Whether a text child or an `{expr}` interpolation reaches this instance —
    /// the content a member list cannot record, since neither has a tag.
    content: bool,
}

impl AstInstance {
    /// Describes `element`, defaulting any part with no recorded position to
    /// `offset` (the element's own).
    pub(super) fn of(element: &Element, offset: u32) -> Self {
        let mut instance = Self {
            tag: element.tag_name().unwrap_or_default(),
            attributes: Vec::new(),
            attribute_offsets: Vec::new(),
            members: Vec::new(),
            member_offsets: Vec::new(),
            content: false,
        };
        for attribute in element.attributes() {
            let event = attribute.is_event();
            let name = if event {
                event_local_name(&attribute)
            } else {
                attribute.name().unwrap_or_default()
            };
            instance.attributes.push((name, event));
            instance
                .attribute_offsets
                .push(attribute.syntax().offset().unwrap_or(offset));
        }
        instance.collect_members(element, MemberArity::Once, offset);
        instance
    }

    #[doc = include_str!("../../../docs/api/validation/contract_validator.doc/collect_members.md")]
    fn collect_members(&mut self, element: &Element, arity: MemberArity, offset: u32) {
        for child in element.children() {
            if matches!(child.kind(), SyntaxKind::Expression)
                || (child.kind() == SyntaxKind::TextNode && !child.text().trim().is_empty())
            {
                self.content = true;
            }
            let Some(child) = Element::cast(child) else {
                continue;
            };
            let Some(tag) = child.tag_name() else {
                continue;
            };
            let child_offset = child.syntax().offset().unwrap_or(offset);
            if child.is_conditional() {
                self.collect_members(
                    &child,
                    degrade(arity, MemberArity::AtMostOnce),
                    child_offset,
                );
                continue;
            }
            if child.is_loop() {
                self.collect_members(&child, MemberArity::Any, child_offset);
                continue;
            }
            if arity == MemberArity::Any
                && matches!(tag.as_str(), "Empty" | "Loading" | "Separator")
            {
                self.collect_members(&child, MemberArity::Any, child_offset);
                continue;
            }
            let slot = child
                .attribute("slot")
                .map(|attribute| attribute.string_value().map_or(Slot::Dynamic, Slot::Named));
            self.members.push((tag, slot, arity));
            self.member_offsets.push(child_offset);
        }
    }

    /// Borrows this instance as the plain [`ElementUse`] the rule takes.
    pub(super) fn as_use(&self) -> ElementUse<'_> {
        ElementUse {
            tag: &self.tag,
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
                    slot: slot.as_ref().map(|slot| match slot {
                        Slot::Named(name) => SlotUse::Named(name),
                        Slot::Dynamic => SlotUse::Dynamic,
                    }),
                    arity: *arity,
                })
                .collect(),
            content: self.content,
        }
    }

    /// Resolves an anchor to the byte offset its diagnostic is reported at.
    pub(super) fn offset_of(&self, anchor: ViolationAnchor, element: u32) -> u32 {
        match anchor {
            ViolationAnchor::Element => element,
            ViolationAnchor::Attribute(index) => self
                .attribute_offsets
                .get(index)
                .copied()
                .unwrap_or(element),
            ViolationAnchor::Member(index) => {
                self.member_offsets.get(index).copied().unwrap_or(element)
            }
        }
    }
}

#[doc = include_str!("../../../docs/api/validation/contract_validator.doc/degrade.md")]
const fn degrade(arity: MemberArity, inner: MemberArity) -> MemberArity {
    match arity {
        MemberArity::Any => MemberArity::Any,
        MemberArity::Once | MemberArity::AtMostOnce => inner,
    }
}
