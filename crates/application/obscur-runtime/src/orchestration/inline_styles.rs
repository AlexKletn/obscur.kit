#![doc = include_str!("../../docs/api/orchestration/inline_styles.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_diagnostics::DiagnosticSink;
use obscur_identity::NodeId;
use obscur_style::cascade::{CascadeOrigin, CascadePriority, ResolvedDeclarations};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::InlineStyle;
use obscur_style::selector::SelectorSpecificity;
use obscur_tree::tree::RetainedTree;

use super::{InlineStyleOutcome, StyleInputs};

#[doc = include_str!("../../docs/api/orchestration/inline_styles.doc/InlineStyleCompiler.md")]
pub type InlineStyleCompiler = Box<dyn Fn(&str) -> InlineStyleOutcome>;

/// The installed compiler and what it has already answered.
#[derive(Default)]
pub(crate) struct InlineStyles {
    /// The port, if one was installed.
    compiler: Option<InlineStyleCompiler>,
    /// One compiled block per distinct `style` string.
    compiled: HashMap<Box<str>, InlineStyle>,
}

impl std::fmt::Debug for InlineStyles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineStyles")
            .field("installed", &self.compiler.is_some())
            .field("compiled", &self.compiled.len())
            .finish()
    }
}

impl InlineStyles {
    /// An empty cache with no compiler installed.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Installs the compiler, discarding whatever the previous one answered — a stale
    /// answer is worse than none.
    pub(crate) fn install(&mut self, compiler: InlineStyleCompiler) {
        // Whatever was compiled by the old one is not this one's answer.
        self.compiled.clear();
        self.compiler = Some(compiler);
    }

    /// Whether a compiler was installed at all.
    pub(crate) fn is_installed(&self) -> bool {
        self.compiler.is_some()
    }

    #[doc = include_str!("../../docs/api/orchestration/inline_styles.doc/block_for.md")]
    pub(crate) fn block_for(
        &mut self,
        text: &str,
        diagnostics: &dyn DiagnosticSink,
    ) -> Option<InlineStyle> {
        let compiler = self.compiler.as_ref()?;
        if let Some(block) = self.compiled.get(text) {
            return Some(block.clone());
        }
        // Compiled once per distinct string, not once per node: a virtualized
        // list of a thousand rows carrying the same `style` pays for one parse,
        // and every row after the first shares the refcounted result.
        let (block, faults) = compiler(text).into_parts();
        // Reported on the same edge that compiles, so a fault is raised once per
        // distinct attribute rather than once per node per frame — the reason a
        // misspelled property in a list row does not become a thousand
        // diagnostics a frame.
        for fault in &faults {
            diagnostics.emit(fault);
        }
        self.compiled.insert(text.into(), block.clone());
        Some(block)
    }

    /// Folds every node's inline style into freshly matched declarations on a full
    /// restyle and prunes compiled blocks no longer referenced by the tree.
    /// Incremental restyles reapply inline declarations only where matching changed.
    pub(crate) fn apply_all(
        &mut self,
        tree: &RetainedTree,
        inputs: &mut StyleInputs,
        diagnostics: &dyn DiagnosticSink,
    ) {
        if !self.is_installed() {
            return;
        }
        let mut live = HashSet::new();
        for (node, retained) in tree.iter() {
            if let Some(text) = retained.attributes().inline_style() {
                live.insert(text);
                self.apply_node(node, text, inputs, diagnostics);
            }
        }
        self.compiled.retain(|text, _| live.contains(text.as_ref()));
    }

    pub(crate) fn apply_invalidated(
        &mut self,
        tree: &RetainedTree,
        inputs: &mut StyleInputs,
        diagnostics: &dyn DiagnosticSink,
        dirty: &StyleInvalidation,
        reconciled: bool,
    ) {
        if !self.is_installed() {
            return;
        }
        for (&node, _) in dirty {
            if let Some(text) = tree
                .get(node)
                .and_then(|node| node.attributes().inline_style())
            {
                self.apply_node(node, text, inputs, diagnostics);
            }
        }
        if reconciled && !self.compiled.is_empty() {
            self.retain_live(tree);
        }
    }

    fn apply_node(
        &mut self,
        node: NodeId,
        text: &str,
        inputs: &mut StyleInputs,
        diagnostics: &dyn DiagnosticSink,
    ) {
        let Some(block) = self.block_for(text, diagnostics) else {
            return;
        };
        if block.is_empty() {
            return;
        }

        let mut resolved = inputs.cascaded_for(node).clone();
        apply(&mut resolved, &block);
        inputs.set_cascaded(node, resolved);
        if !block.custom_properties().is_empty() {
            // Inline custom properties win over the declarations from sheets.
            let mut custom = inputs.custom_for(node).clone();
            for property in block.custom_properties() {
                custom.insert(property.name(), property.value().clone());
            }
            inputs.set_custom(node, custom);
        }
    }

    fn retain_live(&mut self, tree: &RetainedTree) {
        // Dynamic inline attributes can produce a fresh string on every edit.
        let live: HashSet<&str> = tree
            .iter()
            .filter_map(|(_, node)| node.attributes().inline_style())
            .collect();
        self.compiled.retain(|text, _| live.contains(text.as_ref()));
    }

    /// How many distinct strings have been compiled.
    #[cfg(test)]
    pub(crate) fn cached(&self) -> usize {
        self.compiled.len()
    }
}

#[doc = include_str!("../../docs/api/orchestration/inline_styles.doc/apply.md")]
pub(crate) fn apply(resolved: &mut ResolvedDeclarations, block: &InlineStyle) {
    for (order, declaration) in block.declarations().iter().enumerate() {
        let priority = CascadePriority::new(
            CascadeOrigin::Inline,
            declaration.important_flag(),
            // No selector matched, so there is no specificity to speak of. An
            // inline declaration outranks a stylesheet by ORIGIN, not by
            // pretending to a specificity no author could out-write — which is
            // exactly what §4.14 means by an origin that does not artificially
            // inflate specificity.
            SelectorSpecificity::new(0, 0, 0),
            u32::try_from(order).unwrap_or(u32::MAX),
        );
        // Only when it actually wins. An inline NORMAL declaration must not
        // displace an authored `!important` one: §4.14 puts every important
        // declaration above every normal one, whatever origin either came from.
        if resolved
            .priority_of(declaration.property())
            .is_some_and(|winner| priority < winner)
        {
            continue;
        }
        resolved.insert_with_priority(
            declaration.property(),
            declaration.value().clone(),
            priority,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{InlineStyles, apply};
    use crate::orchestration::InlineStyleOutcome;
    use obscur_diagnostics::{
        Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
        NullDiagnosticSink,
    };
    use obscur_identity::FastMap;
    use obscur_style::cascade::{
        CascadeOrigin, CascadePriority, ImportantFlag, ResolvedDeclarations,
    };
    use obscur_style::matching::{BlockDeclaration, InlineStyle};
    use obscur_style::property::{PropertyId, PropertyValue};
    use obscur_style::selector::SelectorSpecificity;

    #[derive(Default)]
    struct CountingSink(std::cell::Cell<usize>);

    impl DiagnosticSink for CountingSink {
        fn emit(&self, _diagnostic: &Diagnostic) {
            self.0.set(self.0.get() + 1);
        }
    }

    fn value(n: f32) -> PropertyValue {
        PropertyValue::Number(n)
    }

    fn from_sheet(
        origin: CascadeOrigin,
        important: ImportantFlag,
        specificity: (u16, u16, u16),
    ) -> ResolvedDeclarations {
        let mut resolved = ResolvedDeclarations::new();
        resolved.insert_with_priority(
            PropertyId::Opacity,
            value(1.0),
            CascadePriority::new(
                origin,
                important,
                SelectorSpecificity::new(specificity.0, specificity.1, specificity.2),
                0,
            ),
        );
        resolved
    }

    fn inline(important: ImportantFlag) -> InlineStyle {
        InlineStyle::new(vec![BlockDeclaration::new(
            PropertyId::Opacity,
            value(2.0),
            important,
        )])
    }

    fn opacity(resolved: &ResolvedDeclarations) -> f32 {
        match resolved.get(PropertyId::Opacity) {
            Some(PropertyValue::Number(n)) => *n,
            other => panic!("expected a number, got {other:?}"),
        }
    }

    #[test]
    fn inline_beats_a_stylesheet_however_specific_its_selector() {
        // The whole point of an origin: no selector an author can write reaches
        // it. A thousand ids would not.
        let mut resolved = from_sheet(
            CascadeOrigin::ComponentModule,
            ImportantFlag::Normal,
            (99, 99, 99),
        );
        apply(&mut resolved, &inline(ImportantFlag::Normal));
        assert_eq!(opacity(&resolved), 2.0);
    }

    #[test]
    fn an_authored_important_declaration_outranks_a_normal_inline_one() {
        // §4.14: every important declaration is above every normal one, whatever
        // origin either came from. Getting this backwards would make `!important`
        // unusable — the one escape hatch an author has would lose to the markup
        // it was written to overrule.
        let mut resolved = from_sheet(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Important,
            (0, 0, 1),
        );
        apply(&mut resolved, &inline(ImportantFlag::Normal));
        assert_eq!(opacity(&resolved), 1.0, "the sheet's !important held");
    }

    #[test]
    fn an_important_inline_declaration_outranks_an_important_authored_one() {
        let mut resolved = from_sheet(
            CascadeOrigin::ComponentModule,
            ImportantFlag::Important,
            (99, 99, 99),
        );
        apply(&mut resolved, &inline(ImportantFlag::Important));
        assert_eq!(opacity(&resolved), 2.0);
    }

    #[test]
    fn a_runtime_override_still_wins() {
        // Origin 6 is above inline, and it is the layer an inspector and an
        // animation write through. Losing to markup would make both useless.
        let mut resolved = from_sheet(
            CascadeOrigin::RuntimeOverride,
            ImportantFlag::Normal,
            (0, 0, 0),
        );
        apply(&mut resolved, &inline(ImportantFlag::Normal));
        assert_eq!(opacity(&resolved), 1.0);
    }

    #[test]
    fn a_property_no_sheet_set_is_simply_taken() {
        let mut resolved = ResolvedDeclarations::new();
        apply(&mut resolved, &inline(ImportantFlag::Normal));
        assert_eq!(opacity(&resolved), 2.0);
    }

    #[test]
    fn the_later_of_two_declarations_in_one_block_wins() {
        // Source order inside the block, which is the only thing left to decide
        // a tie when neither origin nor specificity differ.
        let mut resolved = ResolvedDeclarations::new();
        apply(
            &mut resolved,
            &InlineStyle::new(vec![
                BlockDeclaration::normal(PropertyId::Opacity, value(1.0)),
                BlockDeclaration::normal(PropertyId::Opacity, value(2.0)),
            ]),
        );
        assert_eq!(opacity(&resolved), 2.0);
    }

    #[test]
    fn nothing_is_compiled_until_a_compiler_is_installed() {
        // A `UiRuntime` driven directly, with no CSS front end wired to it, must
        // not silently invent declarations.
        let mut styles = InlineStyles::new();
        assert!(!styles.is_installed());
        assert!(styles.block_for("color:red", &NullDiagnosticSink).is_none());
    }

    #[test]
    fn one_string_is_compiled_once_however_many_nodes_carry_it() {
        // A virtualized list of a thousand rows sharing one `style` pays for one
        // parse, which is the difference between a cache and a parser in the
        // frame loop.
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = std::rc::Rc::clone(&calls);
        let mut styles = InlineStyles::new();
        styles.install(Box::new(move |_| {
            counter.set(counter.get() + 1);
            InlineStyleOutcome::new(InlineStyle::empty())
        }));

        for _ in 0..1000 {
            assert!(
                styles
                    .block_for("height:32px", &NullDiagnosticSink)
                    .is_some()
            );
        }
        assert_eq!(calls.get(), 1);
        assert_eq!(styles.cached(), 1);
    }

    #[test]
    fn a_fault_is_reported_once_per_distinct_attribute_not_once_per_node() {
        // The cost of getting this wrong is a console that scrolls a thousand
        // copies of one message every frame, which is the same as no console.
        let sink = CountingSink::default();
        let mut styles = InlineStyles::new();
        styles.install(Box::new(|_| {
            InlineStyleOutcome::new(InlineStyle::empty()).with_faults(vec![Diagnostic::new(
                DiagnosticCode::new("css.invalid_value"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Style,
                "width: nope",
            )])
        }));

        for _ in 0..100 {
            let _ = styles.block_for("width: nope", &sink);
        }
        assert_eq!(sink.0.get(), 1);
    }

    #[test]
    fn a_string_no_node_carries_any_more_leaves_the_cache() {
        // An application that computes its attribute — a dragged panel writing
        // `style="width: 412px"` every frame — must not leave one live entry per
        // pixel it passed through. The sweep is what makes this a cache and not
        // a leak, and only a walk of the real tree can see it.
        use crate::orchestration::StyleInputs;
        use obscur_identity::ComponentId;
        use obscur_tree::node::NodeKind;
        use obscur_tree::tree::RetainedTree;
        use obscur_view::view::{STYLE, StyleAttributes, ViewProperty, ViewValue};

        let mut tree = RetainedTree::new();
        let node = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.set_root(node);

        let mut styles = InlineStyles::new();
        styles.install(Box::new(|_| {
            InlineStyleOutcome::new(inline(ImportantFlag::Normal))
        }));
        let mut inputs = StyleInputs::new();

        for width in 0..50 {
            let text = format!("width: {width}px");
            let prop = ViewProperty::new(STYLE, ViewValue::Text(text.into()));
            tree.get_mut(node)
                .expect("just allocated")
                .set_attributes(StyleAttributes::extract(&[prop]));
            styles.apply_all(&tree, &mut inputs, &NullDiagnosticSink);
            assert_eq!(styles.cached(), 1, "one live string, one entry");
        }
    }

    #[test]
    fn a_custom_property_written_on_the_element_outranks_the_sheets() {
        // §4.14 origin 5: what an author wrote ON the element wins, and a custom
        // property is a declaration like any other. The properties the sheets
        // set that the attribute does not mention are left alone — an inline
        // `--brand` overrides `--brand`, not the whole theme.
        use crate::orchestration::StyleInputs;
        use obscur_identity::{ComponentId, InternedSymbol};
        use obscur_style::property::CustomProperty;
        use obscur_tree::node::NodeKind;
        use obscur_tree::tree::RetainedTree;
        use obscur_view::view::{STYLE, StyleAttributes, ViewProperty, ViewValue};

        let brand = InternedSymbol::named("--brand");
        let accent = InternedSymbol::named("--accent");

        let mut tree = RetainedTree::new();
        let node = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.set_root(node);
        let prop = ViewProperty::new(STYLE, ViewValue::Text("--brand: 2".into()));
        tree.get_mut(node)
            .expect("just allocated")
            .set_attributes(StyleAttributes::extract(&[prop]));

        let mut styles = InlineStyles::new();
        styles.install(Box::new(move |_| {
            InlineStyleOutcome::new(
                InlineStyle::empty()
                    .with_custom_properties(vec![CustomProperty::new(brand, value(2.0))]),
            )
        }));

        let mut inputs = StyleInputs::new();
        let mut from_sheets = FastMap::default();
        from_sheets.insert(brand, value(1.0));
        from_sheets.insert(accent, value(9.0));
        inputs.set_custom(node, from_sheets);

        styles.apply_all(&tree, &mut inputs, &NullDiagnosticSink);

        assert_eq!(inputs.custom_for(node).get(&brand), Some(&value(2.0)));
        assert_eq!(
            inputs.custom_for(node).get(&accent),
            Some(&value(9.0)),
            "the sheet's other custom property is untouched"
        );
    }

    #[test]
    fn installing_a_new_compiler_forgets_the_old_ones_answers() {
        let mut styles = InlineStyles::new();
        styles.install(Box::new(|_| InlineStyleOutcome::new(InlineStyle::empty())));
        let _ = styles.block_for("color:red", &NullDiagnosticSink);
        assert_eq!(styles.cached(), 1);

        styles.install(Box::new(|_| {
            InlineStyleOutcome::new(inline(ImportantFlag::Normal))
        }));
        assert_eq!(styles.cached(), 0, "a stale answer is worse than none");
        let block = styles
            .block_for("color:red", &NullDiagnosticSink)
            .expect("installed");
        assert!(!block.is_empty(), "the new compiler answered");
    }
}
