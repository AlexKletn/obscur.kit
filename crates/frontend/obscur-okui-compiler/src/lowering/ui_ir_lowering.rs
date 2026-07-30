//! `UiIrLowering`: lowers a `.okui` [`Document`] AST into a [`UiIrDocument`]
//! (spec §2.15).

use obscur_capability::CapabilityId;
use obscur_okui_syntax::ast::{Attribute, Document, Element, Expression};
use obscur_okui_syntax::syntax::{SyntaxKind, SyntaxNode};
use obscur_ui_ir::document::{
    UiIrDocument, UiIrEvent, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrProperty, UiIrValue,
};
use obscur_ui_ir::version::UiIrVersion;

use crate::lowering::{ExpressionLowering, SourceMapBuilder};

/// Why lowering a `.okui` document into UI IR failed, or a non-fatal lowering
/// diagnostic (spec §2.15).
///
/// [`EmptyDocument`](Self::EmptyDocument) and [`MissingTagName`](Self::MissingTagName)
/// are *fatal*: they prevent building a tree and surface as the `Err` of
/// [`UiIrLowering::lower`]. [`MissingLoopKey`](Self::MissingLoopKey) is *non-fatal*:
/// the `<For>` is still lowered (keyless), but the missing mandatory key is
/// reported through the lowering's accumulated diagnostics (spec §2.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweringError {
    /// The document had no top-level elements to lower.
    EmptyDocument,
    /// An element carried no tag name; the byte offset locates it in the source.
    MissingTagName(u32),
    /// A `<For>`/`<VirtualFor>` loop lacked the mandatory stable `key={…}`
    /// (spec §2.11); the byte offset locates it in the source.
    MissingLoopKey(u32),
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => write!(f, "cannot lower an empty document"),
            Self::MissingTagName(offset) => {
                write!(f, "element is missing a tag name at offset {offset}")
            }
            Self::MissingLoopKey(offset) => {
                write!(f, "loop is missing a mandatory `key` at offset {offset}")
            }
        }
    }
}

impl std::error::Error for LoweringError {}

/// Lowers a parsed `.okui` [`Document`] into a versioned [`UiIrDocument`]
/// (spec §2.15).
///
/// # Algorithm
///
/// The pass walks the top-level [`elements`](Document::elements) of the
/// document and turns each into a [`UiIrNode`]:
///
/// - **Root shape.** A document with exactly one top-level element lowers that
///   element as the root; several top-level elements are wrapped in a
///   [`fragment`](UiIrNode::fragment) root, each lowered as a child. An empty
///   document is a [`LoweringError::EmptyDocument`].
/// - **Node kind.** An element whose tag starts with an uppercase letter becomes
///   a [`component`](UiIrNode::component); otherwise an
///   [`element`](UiIrNode::element). The element-shaped
///   [`Conditional`](Element::is_conditional) (`<If>`/`<ElseIf>`/`<Else>`) and
///   [`loop`](Element::is_loop) (`<For>`/`<VirtualFor>`) constructs lower to the
///   dedicated [`Conditional`](UiIrNode::conditional)/[`Loop`](UiIrNode::loop_node)
///   IR kinds (spec §2.11): a loop carries its mandatory `key={…}` on the node
///   (a missing key is a [`LoweringError::MissingLoopKey`] diagnostic) and its
///   `each={…}` collection as a property; a conditional carries its `cond={…}`
///   as a property.
/// - **Attributes.** An event attribute (`on:click=…`) becomes a
///   [`UiIrEvent`] wiring the local event name (after the `:`) to its handler;
///   every other attribute becomes a [`UiIrProperty`] whose value is the lowered
///   expression, string literal, or `Null`. A `requires="cap"` attribute is
///   additionally collected as a declared capability (the §2.13 convention, as
///   in `CapabilityValidator`).
/// - **Children.** Element/conditional/loop children recurse; a non-blank
///   [`TextNode`](SyntaxKind::TextNode) becomes a [`text`](UiIrNode::text) child
///   (blank text is skipped); an [`Expression`](SyntaxKind::Expression)
///   interpolation becomes a synthetic `Expression` element child carrying the
///   lowered expression under a `value` property.
/// - **Ids and source map.** Every node is assigned a stable id from a counter
///   starting at `1`, and each id is recorded against its source byte offset in
///   the [`SourceMapBuilder`].
#[derive(Debug, Clone)]
pub struct UiIrLowering {
    source_map: SourceMapBuilder,
    expr: ExpressionLowering,
    next_id: u64,
    required_capabilities: Vec<Box<str>>,
    diagnostics: Vec<LoweringError>,
}

impl UiIrLowering {
    /// Creates a lowering pass whose source map records spans under
    /// `source_name` (the logical `.okui` source identifier).
    #[must_use]
    pub fn new(source_name: impl Into<Box<str>>) -> Self {
        Self {
            source_map: SourceMapBuilder::new(source_name),
            expr: ExpressionLowering::new(),
            next_id: 1,
            required_capabilities: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Allocates the next stable node id (post-increment).
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Lowers `doc` into a complete [`UiIrDocument`], consuming the pass and
    /// appending any non-fatal lowering diagnostics (e.g. a `<For>` missing its
    /// mandatory key, spec §2.11) to `diagnostics`.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringError::EmptyDocument`] when the document has no
    /// top-level elements, or [`LoweringError::MissingTagName`] when an element
    /// carries no tag name. These are fatal — no document is produced; a
    /// [`LoweringError::MissingLoopKey`] is *not* fatal and is reported through
    /// `diagnostics` while the document is still lowered.
    pub fn lower(
        mut self,
        doc: &Document,
        diagnostics: &mut Vec<LoweringError>,
    ) -> Result<UiIrDocument, LoweringError> {
        let elements: Vec<Element> = doc.elements().collect();

        let root = match elements.as_slice() {
            [] => return Err(LoweringError::EmptyDocument),
            [only] => self.lower_element(only)?,
            many => {
                let id = self.alloc_id();
                self.source_map
                    .record(id, doc.syntax().offset().unwrap_or(0));
                let mut fragment = UiIrNode::fragment(id);
                for element in many {
                    fragment = fragment.with_child(self.lower_element(element)?);
                }
                fragment
            }
        };

        let mut document = UiIrDocument::new(root)
            .with_version(UiIrVersion::CURRENT)
            .with_source_map(self.source_map.build());
        for capability in self.required_capabilities {
            document = document.requiring(CapabilityId::new(capability));
        }
        diagnostics.append(&mut self.diagnostics);
        Ok(document)
    }

    /// Lowers a single element (recursively) into a [`UiIrNode`].
    ///
    /// The element-shaped `<For>`/`<VirtualFor>` and `<If>`/`<ElseIf>`/`<Else>`
    /// control-flow constructs (spec §2.11) lower to the dedicated
    /// [`Loop`](obscur_ui_ir::document::UiIrNodeKind::Loop) and
    /// [`Conditional`](obscur_ui_ir::document::UiIrNodeKind::Conditional) IR
    /// kinds; every other element lowers to a component or element node.
    fn lower_element(&mut self, element: &Element) -> Result<UiIrNode, LoweringError> {
        let offset = element.syntax().offset().unwrap_or(0);
        let id = self.alloc_id();
        self.source_map.record(id, offset);

        let tag = element
            .tag_name()
            .ok_or(LoweringError::MissingTagName(offset))?;

        if element.is_loop() {
            return self.lower_loop(element, id, tag, offset);
        }
        if element.is_conditional() {
            return self.lower_conditional(element, id, tag);
        }

        let mut node = if starts_uppercase(&tag) {
            UiIrNode::component(id, tag)
        } else {
            UiIrNode::element(id, tag)
        };

        node = self.lower_attributes(element, node);
        node = self.lower_children(element, node)?;
        Ok(node)
    }

    /// Lowers a `<For>`/`<VirtualFor>` into a [`Loop`](obscur_ui_ir::document::UiIrNodeKind::Loop)
    /// node, carrying the mandatory `key={…}` on the node and the iterated
    /// collection (`each={…}`) as a property; a missing key is a non-fatal
    /// [`LoweringError::MissingLoopKey`] diagnostic (spec §2.11).
    fn lower_loop(
        &mut self,
        element: &Element,
        id: u64,
        tag: String,
        offset: u32,
    ) -> Result<UiIrNode, LoweringError> {
        let mut node = UiIrNode::loop_node(id, tag);
        if let Some(attribute) = element.attribute("key") {
            let key = if let Some(expression) = attribute.value_expression() {
                self.expr.lower(&expression)
            } else if let Some(literal) = attribute.string_value() {
                UiIrExpression::literal(UiIrValue::Text(literal.into()))
            } else {
                UiIrExpression::literal(UiIrValue::Null)
            };
            node = node.with_key(key);
        } else {
            self.diagnostics.push(LoweringError::MissingLoopKey(offset));
        }
        node = self.lower_attributes(element, node);
        node = self.lower_children(element, node)?;
        Ok(node)
    }

    /// Lowers an `<If>`/`<ElseIf>`/`<Else>` into a
    /// [`Conditional`](obscur_ui_ir::document::UiIrNodeKind::Conditional) node;
    /// the branch condition (`cond={…}`) flows through as a property and the
    /// guarded body as children (spec §2.11).
    fn lower_conditional(
        &mut self,
        element: &Element,
        id: u64,
        tag: String,
    ) -> Result<UiIrNode, LoweringError> {
        let mut node = UiIrNode::conditional(id, tag);
        node = self.lower_attributes(element, node);
        node = self.lower_children(element, node)?;
        Ok(node)
    }

    /// Folds every attribute of `element` onto `node` as an event or property,
    /// collecting any declared `requires` capability along the way.
    fn lower_attributes(&mut self, element: &Element, mut node: UiIrNode) -> UiIrNode {
        for attribute in element.attributes() {
            if attribute.is_event() {
                let event = event_local_name(&attribute);
                let handler = attribute_value_text(&attribute).unwrap_or_default();
                node = node.with_event(UiIrEvent::new(event, handler));
                continue;
            }

            let name = attribute.name().unwrap_or_default();

            // A `<For>`'s mandatory `key` is carried on the Loop node itself
            // (spec §2.11), not duplicated as a property.
            if name == "key" && node.kind() == UiIrNodeKind::Loop {
                continue;
            }

            // A `requires="cap"` string attribute additionally declares a
            // capability requirement (spec §2.13, matching CapabilityValidator).
            if name == "requires"
                && let Some(cap) = attribute.string_value()
            {
                self.required_capabilities.push(cap.into());
            }

            let value = if let Some(expression) = attribute.value_expression() {
                self.expr.lower(&expression)
            } else if let Some(literal) = attribute.string_value() {
                UiIrExpression::literal(UiIrValue::Text(literal.into()))
            } else {
                UiIrExpression::literal(UiIrValue::Null)
            };
            node = node.with_property(UiIrProperty::new(name, value));
        }
        node
    }

    /// Folds every content child of `element` onto `node`.
    fn lower_children(
        &mut self,
        element: &Element,
        mut node: UiIrNode,
    ) -> Result<UiIrNode, LoweringError> {
        for child in element.children() {
            match child.kind() {
                SyntaxKind::Element | SyntaxKind::Conditional | SyntaxKind::ForLoop => {
                    if let Some(child_element) = Element::cast(child) {
                        node = node.with_child(self.lower_element(&child_element)?);
                    }
                }
                SyntaxKind::TextNode => {
                    let text = child.text();
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let id = self.alloc_id();
                        self.source_map.record(id, child.offset().unwrap_or(0));
                        node = node.with_child(UiIrNode::text(id, trimmed));
                    }
                }
                SyntaxKind::Expression => {
                    if let Some(expression) = Expression::cast(child.clone()) {
                        node = node.with_child(self.lower_interpolation(&child, &expression));
                    }
                }
                _ => {}
            }
        }
        Ok(node)
    }

    /// Lowers an `{expr}` interpolation to the IR's
    /// [interpolation node](UiIrNode::interpolation), carrying the expression
    /// itself — `ViewCodegen` emits it as the text node's content.
    fn lower_interpolation(&mut self, node: &SyntaxNode, expression: &Expression) -> UiIrNode {
        let id = self.alloc_id();
        self.source_map.record(id, node.offset().unwrap_or(0));
        UiIrNode::interpolation(id, self.expr.lower(expression))
    }
}

/// Returns whether `tag`'s first character is an ASCII uppercase letter — the
/// component-vs-element distinction.
fn starts_uppercase(tag: &str) -> bool {
    tag.chars().next().is_some_and(char::is_uppercase)
}

/// Returns an event attribute's local name — the identifier after the `:` in
/// `on:click` (`"click"`), falling back to the attribute's own name.
///
/// `pub(crate)` because the validators must name an event exactly as the IR does:
/// a diagnostic about `on:clik` has to quote the string the binding was built
/// from, and the `ContractValidator` used to keep a byte-identical private copy of
/// this function to do it.
pub(crate) fn event_local_name(attribute: &Attribute) -> String {
    let mut seen_colon = false;
    for token in attribute.syntax().tokens() {
        match token.kind() {
            SyntaxKind::Colon => seen_colon = true,
            SyntaxKind::Ident if seen_colon => return token.text().to_owned(),
            _ => {}
        }
    }
    attribute.name().unwrap_or_default()
}

/// Extracts an attribute's value as raw text: a quoted string's inner text, a
/// dynamic expression's source, or a bare `AttributeValue` token's text (as in
/// `on:click=submit`). Returns `None` when the attribute has no value.
///
/// `pub(crate)` because for an **event** attribute this string *is* the handler
/// reference — the name `name_binding::fallback_symbol` hashes into the command
/// symbol the runtime dispatches on. A checker that re-derived it would be
/// checking a name the IR might not carry, and a validator that admits one string
/// while the IR emits another is worse than no validator at all. One extraction,
/// so the checked name and the hashed name cannot drift.
pub(crate) fn attribute_value_text(attribute: &Attribute) -> Option<String> {
    if let Some(literal) = attribute.string_value() {
        return Some(literal);
    }
    if let Some(expression) = attribute.value_expression() {
        return Some(expression.text());
    }
    attribute
        .syntax()
        .child_nodes()
        .find(|n| n.kind() == SyntaxKind::AttributeValue)
        .map(|value| value.text().trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::parser::Parser;
    use obscur_ui_ir::document::UiIrNodeKind;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    fn lower(src: &str) -> Result<UiIrDocument, LoweringError> {
        UiIrLowering::new("test.okui").lower(&document(src), &mut Vec::new())
    }

    /// Lowers `src`, returning the document alongside the accumulated non-fatal
    /// diagnostics.
    fn lower_with_diagnostics(src: &str) -> (UiIrDocument, Vec<LoweringError>) {
        let mut diagnostics = Vec::new();
        let doc = UiIrLowering::new("test.okui")
            .lower(&document(src), &mut diagnostics)
            .expect("lowers");
        (doc, diagnostics)
    }

    #[test]
    fn lowers_component_with_property_event_and_text_child() {
        let doc = lower("<Button label=\"Save\" on:click=submit>Hi</Button>").expect("lowers");
        let root = doc.root();
        assert_eq!(root.kind(), UiIrNodeKind::Component);
        assert_eq!(root.tag(), "Button");

        assert_eq!(
            root.property("label"),
            Some(&UiIrExpression::literal(UiIrValue::Text("Save".into())))
        );

        assert_eq!(root.events().len(), 1);
        assert_eq!(root.events()[0].event(), "click");
        assert_eq!(root.events()[0].handler(), "submit");

        assert_eq!(root.children().len(), 1);
        let text = &root.children()[0];
        assert_eq!(text.kind(), UiIrNodeKind::Text);
        assert_eq!(text.content(), "Hi");
    }

    #[test]
    fn dynamic_property_lowers_to_binding() {
        let doc = lower("<T x={a.b}/>").expect("lowers");
        assert_eq!(
            doc.root().property("x"),
            Some(&UiIrExpression::binding("a.b"))
        );
    }

    #[test]
    fn requires_attribute_collects_capability() {
        let doc = lower("<Cam requires=\"camera\"/>").expect("lowers");
        assert!(doc.requires(&CapabilityId::new("camera")));
    }

    #[test]
    fn multi_root_document_becomes_fragment() {
        let doc = lower("<A/><B/>").expect("lowers");
        let root = doc.root();
        assert_eq!(root.kind(), UiIrNodeKind::Fragment);
        assert_eq!(root.children().len(), 2);
        assert!(
            root.children()
                .iter()
                .all(|c| c.kind() == UiIrNodeKind::Component)
        );
    }

    #[test]
    fn lowercase_tag_is_an_element() {
        let doc = lower("<div/>").expect("lowers");
        assert_eq!(doc.root().kind(), UiIrNodeKind::Element);
    }

    #[test]
    fn interpolation_becomes_an_interpolation_node() {
        let doc = lower("<T>{count}</T>").expect("lowers");
        let child = &doc.root().children()[0];
        assert_eq!(
            child.interpolated_value(),
            Some(&UiIrExpression::binding("count"))
        );
    }

    #[test]
    fn empty_document_is_an_error() {
        assert_eq!(lower(""), Err(LoweringError::EmptyDocument));
    }

    #[test]
    fn for_loop_lowers_to_loop_node_carrying_key() {
        let (doc, diagnostics) =
            lower_with_diagnostics("<For each={items} key={row.id}><Row/></For>");
        let root = doc.root();
        assert_eq!(root.kind(), UiIrNodeKind::Loop);
        assert_eq!(root.tag(), "For");
        // The mandatory key is carried on the node, not as a property.
        assert_eq!(root.key(), Some(&UiIrExpression::binding("row.id")));
        assert!(root.property("key").is_none());
        // The iterated collection flows through as a property.
        assert_eq!(
            root.property("each"),
            Some(&UiIrExpression::binding("items"))
        );
        // The body is a child.
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].tag(), "Row");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn for_loop_without_key_reports_missing_key_diagnostic() {
        let (doc, diagnostics) = lower_with_diagnostics("<For each={items}><Row/></For>");
        assert_eq!(doc.root().kind(), UiIrNodeKind::Loop);
        assert!(doc.root().key().is_none());
        assert!(
            diagnostics
                .iter()
                .any(|d| matches!(d, LoweringError::MissingLoopKey(_))),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn if_lowers_to_conditional_node_with_condition_property() {
        let doc = lower("<If cond={open}><A/></If>").expect("lowers");
        let root = doc.root();
        assert_eq!(root.kind(), UiIrNodeKind::Conditional);
        assert_eq!(root.tag(), "If");
        assert_eq!(
            root.property("cond"),
            Some(&UiIrExpression::binding("open"))
        );
        assert_eq!(root.children().len(), 1);
    }

    #[test]
    fn virtual_for_lowers_to_loop_node_tagged_virtual_for() {
        // Lowering is not where a `<VirtualFor>` is refused — `Compiler::compile`
        // is (`CollectionValidator`, §2.11), and its diagnostics accumulate
        // rather than stopping the pass. So this pass still classifies the tag,
        // and reaching this IR at all means someone ignored the compile's
        // `has_errors()` — the same contract every other rule here has.
        let doc = lower("<VirtualFor each={rows} key={id}><R/></VirtualFor>").expect("lowers");
        assert_eq!(doc.root().kind(), UiIrNodeKind::Loop);
        assert_eq!(doc.root().tag(), "VirtualFor");
    }

    #[test]
    fn source_map_records_root_offset() {
        let doc = lower("<Button/>").expect("lowers");
        assert!(!doc.source_map().is_empty());
        assert!(doc.source_map().get(doc.root().id()).is_some());
    }
}
