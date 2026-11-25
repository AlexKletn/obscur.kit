use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::{ScreenData, ScreenRow, screen_binding::ScreenBinding};
use crate::identity::InternedSymbol;
use crate::okui_compiler::bridge::UiIrViewBridge;
use crate::ui_ir::document::{UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use crate::ui_ir::name_binding;
use crate::view::view::{
    DEFAULT_VIRTUAL_ROW_EXTENT, ViewFragment, ViewKey, ViewNode, ViewProperty, ViewValue,
};

const VIRTUAL_FOR: &str = "VirtualFor";

#[derive(Clone)]
pub(super) struct ScreenLowering {
    bindings: Arc<HashMap<Box<str>, ScreenBinding>>,
    dynamic: Arc<HashSet<u64>>,
    scope: String,
    rows: Vec<(Box<str>, Arc<[ScreenRow]>, usize)>,
}

impl ScreenLowering {
    pub(super) fn new(
        bindings: Arc<HashMap<Box<str>, ScreenBinding>>,
        dynamic: Arc<HashSet<u64>>,
        scope: u32,
    ) -> Self {
        Self {
            bindings,
            dynamic,
            scope: format!("okui-screen:{scope}"),
            rows: Vec::new(),
        }
    }

    fn lookup(&self, path: &str) -> Option<ViewValue> {
        if path == "index"
            && let Some((_, _, index)) = self.rows.last()
        {
            return i64::try_from(*index).ok().map(ViewValue::Int);
        }
        if let Some((alias, field)) = path.split_once('.')
            && let Some((_, rows, index)) =
                self.rows.iter().rev().find(|(name, _, _)| &**name == alias)
        {
            return rows[*index].get(field).cloned();
        }
        self.bindings.get(path).and_then(ScreenBinding::value)
    }

    fn lower_loop(&self, node: &UiIrNode) -> Vec<ViewNode> {
        if self.boolean_property(node, "loading")
            && let Some(slot) = node
                .children()
                .iter()
                .find(|child| child.tag() == "Loading")
        {
            return self.lower_children(slot);
        }

        let rows = match node.property("each") {
            Some(UiIrExpression::Binding(path)) => match self.bindings.get(path.as_ref()) {
                Some(ScreenBinding::Rows(rows)) => Some(rows),
                _ => None,
            },
            _ if node.tag() == VIRTUAL_FOR => return UiIrViewBridge::node_to_view_nodes(node),
            _ => None,
        };
        let Some(rows) = rows.filter(|rows| !rows.is_empty()) else {
            return node
                .children()
                .iter()
                .find(|child| child.tag() == "Empty")
                .map_or_else(Vec::new, |empty| self.lower_children(empty));
        };
        let alias = match node.property("as") {
            Some(
                UiIrExpression::Binding(alias) | UiIrExpression::Literal(UiIrValue::Text(alias)),
            ) => alias.clone(),
            _ => "item".into(),
        };
        if node.tag() == VIRTUAL_FOR {
            self.lower_virtual(node, rows, alias)
        } else {
            self.lower_for(node, rows, &alias)
        }
    }

    fn lower_for(&self, node: &UiIrNode, rows: &Arc<[ScreenRow]>, alias: &str) -> Vec<ViewNode> {
        let separator = node
            .children()
            .iter()
            .find(|child| child.tag() == "Separator");
        let mut nodes = Vec::new();
        for index in 0..rows.len() {
            let mut lowering = self.clone();
            lowering.rows.push((alias.into(), Arc::clone(rows), index));
            if index > 0
                && let Some(separator) = separator
            {
                nodes.extend(lowering.lower_children(separator));
            }
            nodes.extend(lowering.lower_row(node));
        }
        nodes
    }

    fn lower_virtual(
        &self,
        node: &UiIrNode,
        rows: &Arc<[ScreenRow]>,
        alias: Box<str>,
    ) -> Vec<ViewNode> {
        let rows = Arc::clone(rows);
        let id = InternedSymbol::named(&format!("{}:{}", self.scope, node.id()));
        let separator = node
            .children()
            .iter()
            .find(|child| child.tag() == "Separator")
            .map(|slot| {
                let extent = match slot.property("extent") {
                    Some(UiIrExpression::Literal(UiIrValue::Int(value))) => Some(*value as f32),
                    Some(UiIrExpression::Literal(UiIrValue::Float(value))) => Some(*value as f32),
                    _ => None,
                };
                (extent, ViewFragment::from_nodes(self.lower_children(slot)))
            });
        let node = node.clone();
        let lowering = self.clone();
        let marker = ViewNode::virtual_list_with_rows(
            id,
            rows.len(),
            DEFAULT_VIRTUAL_ROW_EXTENT,
            move |index| {
                let mut lowering = lowering.clone();
                lowering
                    .rows
                    .push((alias.clone(), Arc::clone(&rows), index));
                ViewFragment::from_nodes(lowering.lower_row(&node))
            },
        );
        vec![match separator {
            Some((extent, content)) => marker.with_virtual_separator(extent, content),
            None => marker,
        }]
    }

    fn lower_row(&mut self, node: &UiIrNode) -> Vec<ViewNode> {
        let key = node.key().and_then(|expr| self.key(expr));
        if let Some(key) = &key {
            use std::fmt::Write as _;
            let _ = write!(self.scope, ":{}:{key:?}", node.id());
        }
        let body = node
            .children()
            .iter()
            .filter(|child| !matches!(child.tag(), "Empty" | "Loading" | "Separator"));
        self.lower_nodes(body)
            .into_iter()
            .map(|row| match &key {
                Some(key) if row.key().is_none() => row.with_key(key.clone()),
                _ => row,
            })
            .collect()
    }

    fn key(&self, expression: &UiIrExpression) -> Option<ViewKey> {
        let value = match expression {
            UiIrExpression::Binding(path) => self.lookup(path)?,
            UiIrExpression::Literal(value) => view_value(value),
            UiIrExpression::Call { .. } => return None,
        };
        match value {
            ViewValue::Int(value) => Some(ViewKey::from(value.cast_unsigned())),
            ViewValue::Text(value) => Some(ViewKey::from(value.as_ref())),
            _ => None,
        }
    }

    fn boolean_property(&self, node: &UiIrNode, name: &str) -> bool {
        match node.property(name) {
            Some(UiIrExpression::Binding(path)) => self.lookup(path) == Some(ViewValue::Bool(true)),
            Some(UiIrExpression::Literal(UiIrValue::Bool(value))) => *value,
            _ => false,
        }
    }

    /// Lowers siblings in authored order, selecting at most one branch of each conditional chain.
    pub(super) fn lower_nodes<'a>(
        &self,
        nodes: impl IntoIterator<Item = &'a UiIrNode>,
    ) -> Vec<ViewNode> {
        let mut nodes = nodes.into_iter().peekable();
        let mut output = Vec::new();
        while let Some(node) = nodes.next() {
            if node.kind() != UiIrNodeKind::Conditional || node.tag() != "If" {
                output.extend(self.lower(node));
                continue;
            }
            let mut selected = self.boolean_property(node, "cond").then_some(node);
            while let Some(branch) = nodes.next_if(|next| {
                next.kind() == UiIrNodeKind::Conditional && matches!(next.tag(), "ElseIf" | "Else")
            }) {
                let fallback = branch.tag() == "Else";
                if selected.is_none() && (fallback || self.boolean_property(branch, "cond")) {
                    selected = Some(branch);
                }
                if fallback {
                    break;
                }
            }
            if let Some(branch) = selected {
                output.extend(self.lower_children(branch));
            }
        }
        output
    }

    /// Lowers one IR node, substituting what the data supplies. Returns a `Vec`
    /// for the same reason the bridge does: a transparent node splices.
    fn lower(&self, node: &UiIrNode) -> Vec<ViewNode> {
        if node.kind() == UiIrNodeKind::Loop {
            return self.lower_loop(node);
        }
        // A body `{expr}`: the IR carries the expression, and the bridge — which
        // has nothing to evaluate it with — lowers it to an anonymous element with
        // the expression under a hashed property, i.e. an empty box. This is the
        // one node kind whose lowering is *replaced* rather than repaired.
        if let Some(expression) = node.interpolated_value() {
            return vec![ViewNode::text(self.render(expression))];
        }
        // Nothing in this subtree reads the data, so the bridge's lowering of it is
        // already this frame's answer. This is what keeps a mostly static screen at
        // one bridge call.
        if !self.dynamic.contains(&node.id()) {
            return UiIrViewBridge::node_to_view_nodes(node);
        }
        match node.kind() {
            // A real element: keep the bridge's node, substitute its own bound
            // properties, and re-lower its children through this path so a nested
            // binding is substituted too.
            UiIrNodeKind::Element | UiIrNodeKind::Component => {
                let mut lowered = UiIrViewBridge::node_to_view_nodes(node);
                for view in &mut lowered {
                    self.substitute(node, view);
                }
                lowered
            }
            // Transparent kinds: the bridge splices their children into the
            // surrounding fragment, so the re-lowered children *are* the result.
            // Read from the node's own `kind`, not re-derived from its tag — the
            // front ends already decided which it is.
            UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => {
                self.lower_children(node)
            }
            // Literal content: nothing to substitute (and never marked dynamic —
            // this arm exists so the match stays total rather than by luck).
            UiIrNodeKind::Text => UiIrViewBridge::node_to_view_nodes(node),
        }
    }

    /// Every child of `node`, lowered through [`lower`](Self::lower) and spliced.
    fn lower_children(&self, node: &UiIrNode) -> Vec<ViewNode> {
        self.lower_nodes(node.children())
    }

    /// Replaces `view`'s bound property values and its children with the data-aware
    /// versions. A no-op for a text node, which has neither.
    fn substitute(&self, node: &UiIrNode, view: &mut ViewNode) {
        let ViewNode::Element {
            key,
            props,
            children,
            ..
        } = view
        else {
            return;
        };
        if let Some(expression @ UiIrExpression::Binding(_)) = node.key() {
            *key = self.key(expression);
        }
        for property in node.properties() {
            let UiIrExpression::Binding(path) = property.value() else {
                continue;
            };
            // The symbol the bridge bound this attribute to — through the same
            // tag-aware function it used, so the two cannot disagree about which
            // property to overwrite.
            let symbol = name_binding::attr_symbol_on(node.tag(), property.name());
            let Some(value) = self.lookup(path) else {
                use crate::view::view::{CHECKED, DISABLED, READONLY};
                // The bridge leaves unmatched bindings as text. A path is
                // truthy, so presence-based flags must omit that placeholder.
                if matches!(symbol, CHECKED | DISABLED | READONLY) {
                    props.retain(|prop| prop.name() != symbol);
                }
                continue;
            };
            for existing in props.iter_mut().filter(|prop| prop.name() == symbol) {
                *existing = ViewProperty::new(symbol, value.clone());
            }
        }
        *children = ViewFragment::from_nodes(self.lower_children(node));
    }

    /// What a body `{expr}` renders this frame.
    fn render(&self, expression: &UiIrExpression) -> Box<str> {
        match expression {
            UiIrExpression::Binding(path) => match self.lookup(path) {
                Some(value) => ScreenData::render(&value),
                // Keep the path visible rather than rendering nothing: an unbound
                // binding is a mistake, and a blank line hides it.
                None => path.clone(),
            },
            // Text is its own rendering; the other literals go through the value
            // mapping so one function decides how a number reads.
            UiIrExpression::Literal(UiIrValue::Text(text)) => text.clone(),
            UiIrExpression::Literal(value) => ScreenData::render(&view_value(value)),
            // `{format(x)}` has no evaluator on this route — there are no Rust
            // locals to call anything with. The build-time route emits the call as
            // Rust and is the one that can run it.
            UiIrExpression::Call { .. } => Box::from(""),
        }
    }
}

#[doc = include_str!("../../docs/api/screen/okui_screen.doc/mark_dynamic.md")]
pub(super) fn mark_dynamic(node: &UiIrNode, out: &mut HashSet<u64>) -> bool {
    let mut dynamic = matches!(node.kind(), UiIrNodeKind::Loop | UiIrNodeKind::Conditional)
        || matches!(node.key(), Some(UiIrExpression::Binding(_)))
        || node.interpolated_value().is_some()
        || node
            .properties()
            .iter()
            .any(|property| matches!(property.value(), UiIrExpression::Binding(_)));
    for child in node.children() {
        // Not `||`: every child must be visited, so short-circuiting would leave
        // later subtrees unmarked.
        dynamic |= mark_dynamic(child, out);
    }
    if dynamic {
        out.insert(node.id());
    }
    dynamic
}

#[doc = include_str!("../../docs/api/screen/okui_screen.doc/view_value.md")]
fn view_value(value: &UiIrValue) -> ViewValue {
    match value {
        UiIrValue::Null => ViewValue::Null,
        UiIrValue::Text(text) => ViewValue::Text(text.clone()),
        UiIrValue::Bool(flag) => ViewValue::Bool(*flag),
        UiIrValue::Int(number) => ViewValue::Int(*number),
        UiIrValue::Float(number) => ViewValue::Float(*number),
    }
}
