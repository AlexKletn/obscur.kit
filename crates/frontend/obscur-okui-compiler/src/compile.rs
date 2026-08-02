#![doc = include_str!("compile.doc/_module.md")]

use std::collections::HashMap;

use obscur_capability::CapabilityId;
use obscur_okui_syntax::ast::{Document, Element};
use obscur_ui_ir::document::UiIrDocument;
use obscur_ui_ir::source::SourceMap;
use obscur_view::contract::ComponentSurface;

use crate::lowering::{LoweringError, UiIrLowering};
use crate::resolution::{NameResolver, ResolutionError};
use crate::symbol::{Symbol, SymbolKind, SymbolTable};
use crate::typing::TypeError;
use crate::validation::{
    CapabilityValidator, CollectionValidator, CommandSurface, CommandValidator, ComponentContract,
    ContractValidator, ExpressionValidator, OutletValidator, ValidationDiagnostic,
    default_contracts,
};

#[doc = include_str!("compile.doc/ContractRegistry.md")]
#[derive(Debug, Clone, Default)]
pub struct ContractRegistry {
    contracts: HashMap<Box<str>, ComponentContract>,
}

impl ContractRegistry {
    #[doc = include_str!("compile.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("compile.doc/builtins.md")]
    #[must_use]
    pub fn builtins() -> Self {
        let mut registry = Self::new();
        for contract in default_contracts() {
            registry.register(contract);
        }
        registry
    }

    #[doc = include_str!("compile.doc/with.md")]
    #[must_use]
    pub fn with(mut self, contract: ComponentContract) -> Self {
        self.register(contract);
        self
    }

    #[doc = include_str!("compile.doc/register.md")]
    pub fn register(&mut self, contract: ComponentContract) -> Option<ComponentContract> {
        self.contracts.insert(contract.component().into(), contract)
    }

    #[doc = include_str!("compile.doc/register_surface.md")]
    pub fn register_surface(&mut self, surface: &ComponentSurface) -> Option<ComponentContract> {
        self.register(surface.into())
    }

    #[doc = include_str!("compile.doc/with_surface.md")]
    #[must_use]
    pub fn with_surface(mut self, surface: &ComponentSurface) -> Self {
        self.register_surface(surface);
        self
    }

    #[doc = include_str!("compile.doc/get.md")]
    #[must_use]
    pub fn get(&self, component: &str) -> Option<&ComponentContract> {
        self.contracts.get(component)
    }

    #[doc = include_str!("compile.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    #[doc = include_str!("compile.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }
}

#[doc = include_str!("compile.doc/CompileDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileDiagnostic {
    #[doc = include_str!("compile.doc/Resolution.md")]
    Resolution(ResolutionError),
    #[doc = include_str!("compile.doc/Type.md")]
    Type(TypeError),
    #[doc = include_str!("compile.doc/Validation.md")]
    Validation(ValidationDiagnostic),
    #[doc = include_str!("compile.doc/Lowering.md")]
    Lowering(LoweringError),
}

impl std::fmt::Display for CompileDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolution(error) => write!(f, "{error}"),
            Self::Type(error) => write!(f, "{error}"),
            Self::Validation(diagnostic) => {
                write!(
                    f,
                    "{} at offset {}",
                    diagnostic.message(),
                    diagnostic.offset()
                )
            }
            Self::Lowering(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CompileDiagnostic {}

impl From<ResolutionError> for CompileDiagnostic {
    fn from(error: ResolutionError) -> Self {
        Self::Resolution(error)
    }
}

impl From<TypeError> for CompileDiagnostic {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}

impl From<ValidationDiagnostic> for CompileDiagnostic {
    fn from(diagnostic: ValidationDiagnostic) -> Self {
        Self::Validation(diagnostic)
    }
}

impl From<LoweringError> for CompileDiagnostic {
    fn from(error: LoweringError) -> Self {
        Self::Lowering(error)
    }
}

#[doc = include_str!("compile.doc/CompileResult.md")]
#[derive(Debug, Clone)]
pub struct CompileResult {
    document: Option<UiIrDocument>,
    diagnostics: Vec<CompileDiagnostic>,
    source_map: SourceMap,
}

impl CompileResult {
    #[doc = include_str!("compile.doc/document.md")]
    #[must_use]
    pub fn document(&self) -> Option<&UiIrDocument> {
        self.document.as_ref()
    }

    #[doc = include_str!("compile.doc/diagnostics.md")]
    #[must_use]
    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }

    #[doc = include_str!("compile.doc/source_map.md")]
    #[must_use]
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    #[doc = include_str!("compile.doc/has_errors.md")]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    #[doc = include_str!("compile.doc/into_document.md")]
    #[must_use]
    pub fn into_document(self) -> Option<UiIrDocument> {
        self.document
    }
}

#[doc = include_str!("compile.doc/Compiler.md")]
#[derive(Debug, Clone, Default)]
pub struct Compiler {
    source_name: Box<str>,
    registry: ContractRegistry,
    granted: Vec<CapabilityId>,
    commands: CommandSurface,
    symbols: Vec<Symbol>,
}

impl Compiler {
    #[doc = include_str!("compile.doc/new_2.md")]
    #[must_use]
    pub fn new(source_name: impl Into<Box<str>>) -> Self {
        Self {
            source_name: source_name.into(),
            registry: ContractRegistry::builtins(),
            ..Self::default()
        }
    }

    #[doc = include_str!("compile.doc/with_registry.md")]
    #[must_use]
    pub fn with_registry(mut self, registry: ContractRegistry) -> Self {
        self.registry = registry;
        self
    }

    #[doc = include_str!("compile.doc/granting.md")]
    #[must_use]
    pub fn granting(mut self, capability: CapabilityId) -> Self {
        self.granted.push(capability);
        self
    }

    #[doc = include_str!("compile.doc/claiming_command.md")]
    #[must_use]
    pub fn claiming_command(mut self, name: impl Into<Box<str>>) -> Self {
        self.commands.claim(name);
        self
    }

    #[doc = include_str!("compile.doc/with_commands.md")]
    #[must_use]
    pub fn with_commands(mut self, commands: CommandSurface) -> Self {
        self.commands = commands;
        self
    }

    #[doc = include_str!("compile.doc/defining.md")]
    #[must_use]
    pub fn defining(mut self, symbol: Symbol) -> Self {
        self.symbols.push(symbol);
        self
    }

    #[doc = include_str!("compile.doc/compile.md")]
    #[must_use]
    pub fn compile(&self, document: &Document) -> CompileResult {
        let mut diagnostics = Vec::new();

        // 1. Name resolution against the seeded symbol table.
        let mut table = SymbolTable::new();
        for symbol in &self.symbols {
            table.define(symbol.clone());
        }
        for error in NameResolver::new().resolve(document, &mut table) {
            diagnostics.push(CompileDiagnostic::Resolution(error));
        }

        // 2. Type checking has no whole-document pass yet (see the doc comment).

        // 3. Validation: expressions, capabilities, then contracts. The
        //    expression check is handed the names declared as `Function` symbols
        //    (§2.10's "explicitly exported pure functions"), so a call to one is
        //    admitted where an unconstrained free call is refused.
        let functions: Vec<Box<str>> = self
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind(), SymbolKind::Function))
            .map(|symbol| symbol.name().into())
            .collect();
        for diagnostic in ExpressionValidator::new()
            .with_functions(functions)
            .validate_document(document)
        {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        let capability_validator = CapabilityValidator::new(self.granted.iter().cloned());
        for diagnostic in capability_validator.validate(document) {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        for diagnostic in CommandValidator::new(self.commands.clone()).validate(document) {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        for diagnostic in CollectionValidator::new().validate(document) {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        for diagnostic in OutletValidator::new().validate(document) {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        let contract_validator = ContractValidator::new();
        for element in document.elements() {
            self.validate_contracts(&element, &contract_validator, &mut diagnostics);
        }

        // 4. Lowering to UI IR. Non-fatal lowering diagnostics (e.g. a `<For>`
        //    missing its mandatory key, §2.11) accumulate alongside the produced
        //    document; a fatal error yields no document but keeps every earlier
        //    diagnostic.
        let mut lowering_diagnostics = Vec::new();
        let outcome =
            UiIrLowering::new(self.source_name.clone()).lower(document, &mut lowering_diagnostics);
        for error in lowering_diagnostics {
            diagnostics.push(CompileDiagnostic::Lowering(error));
        }
        match outcome {
            Ok(document) => {
                let source_map = document.source_map().clone();
                CompileResult {
                    document: Some(document),
                    diagnostics,
                    source_map,
                }
            }
            Err(error) => {
                diagnostics.push(CompileDiagnostic::Lowering(error));
                CompileResult {
                    document: None,
                    diagnostics,
                    source_map: SourceMap::new(),
                }
            }
        }
    }

    #[doc = include_str!("compile.doc/validate_contracts.md")]
    fn validate_contracts(
        &self,
        element: &Element,
        validator: &ContractValidator,
        diagnostics: &mut Vec<CompileDiagnostic>,
    ) {
        if let Some(tag) = element.tag_name()
            && let Some(contract) = self.registry.get(&tag)
        {
            for diagnostic in validator.validate_element(element, contract) {
                diagnostics.push(CompileDiagnostic::Validation(diagnostic));
            }
        }
        for child in element.children() {
            if let Some(child_element) = Element::cast(child) {
                self.validate_contracts(&child_element, validator, diagnostics);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::ast::Document;
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    fn button_registry() -> ContractRegistry {
        ContractRegistry::new().with(
            ComponentContract::new("Button")
                .requiring_prop("label")
                .allowing_event("click"),
        )
    }

    #[test]
    fn registry_registers_and_looks_up_contracts() {
        let registry = button_registry();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Button").is_some());
        assert!(registry.get("Missing").is_none());
    }

    #[test]
    fn valid_document_compiles_with_document_and_no_errors() {
        let doc = document("<Button label=\"Save\" on:click=submit/>");
        let result = Compiler::new("app.okui")
            .with_registry(button_registry())
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        let ir = result.document().expect("a lowered document");
        assert_eq!(ir.root().tag(), "Button");
        assert!(!result.source_map().is_empty());
        assert!(result.source_map().get(ir.root().id()).is_some());
    }

    #[test]
    fn the_default_registry_checks_widgets_without_any_registration() {
        // The point of seeding `widget_contracts`: `<Button>` used to compile
        // clean whatever you wrote on it, because no contract was registered and
        // an unknown tag is left unchecked. A DEFAULT compile — no
        // `with_registry` — now catches a typo and a missing required prop.
        let typo = Compiler::new("app.okui").compile(&document(
            "<Button label=\"Save\" laebl=\"Save\" on:click=submit/>",
        ));
        assert!(
            typo.has_errors(),
            "a typo'd Button attribute must not compile clean"
        );

        let missing = Compiler::new("app.okui").compile(&document("<Button on:click=submit/>"));
        assert!(missing.has_errors(), "Button requires its `label`");

        // And the honest form still compiles.
        let ok = Compiler::new("app.okui").compile(&document(
            "<Button label=\"Save\" disabled=\"true\" on:click=submit/>",
        ));
        assert!(!ok.has_errors(), "diagnostics: {:?}", ok.diagnostics());
    }

    #[test]
    fn the_default_registry_leaves_collection_widgets_open() {
        // Menu/Tabs/... are registered with no prop surface precisely so they
        // stay unchecked rather than asserting a vocabulary that does not exist.
        let result = Compiler::new("app.okui").compile(&document("<Tabs anything=\"1\"/>"));
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn a_mounted_base_element_reports_authored_children_rather_than_dropping_them() {
        // The default registry, no `with_registry`. `<textarea>Hello</textarea>` is
        // the case that forced this: HTML makes a textarea's child text its default
        // value, this toolkit spells that `value="…"`, and `BuiltinComponents` now
        // mounts a component whose output replaces the host's children — so that
        // markup would have rendered an EMPTY field. It reports instead.
        let content = Compiler::new("app.okui").compile(&document("<textarea>Hello</textarea>"));
        assert!(content.has_errors(), "an authored value must not vanish");
        // An element child is the same fault: `<select><option/></select>` reaches
        // no tree either, because `SelectProps` reads a comma-separated `options`.
        let child = Compiler::new("app.okui").compile(&document("<select><option/></select>"));
        assert!(child.has_errors(), "diagnostics: {:?}", child.diagnostics());

        // The migration compiles, and so does every prop these tags already took —
        // only the CHILD surface was closed, so nothing else had to change.
        let ok = Compiler::new("app.okui").compile(&document(
            "<div><textarea value=\"Hello\" class=\"ta\" placeholder=\"Notes\" \
             on:input=edit/><input readonly=\"true\"/></div>",
        ));
        assert!(!ok.has_errors(), "diagnostics: {:?}", ok.diagnostics());
    }

    #[test]
    fn a_base_element_that_mounts_nothing_still_takes_its_children() {
        // The other half, and the campaign's line: `<button>` and `<table>` keep
        // their children — `<table>` because nothing mounts on it, `<button>`
        // because what mounts on it renders the §2.12 default-slot outlet and so
        // owns no children of its own. A contract that forbade them would break
        // markup that works today.
        for src in ["<button>Save</button>", "<table><tr/></table>"] {
            let result = Compiler::new("app.okui").compile(&document(src));
            assert!(
                !result.has_errors(),
                "{src} must still compile: {:?}",
                result.diagnostics()
            );
        }
    }

    #[test]
    fn typoed_attribute_is_reported_as_unknown_attribute() {
        let doc = document("<Button label=\"Save\" colr=\"red\"/>");
        let result = Compiler::new("app.okui")
            .with_registry(button_registry())
            .compile(&doc);
        assert!(result.has_errors());
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown attribute 'colr' on component 'Button'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn typoed_event_is_reported_as_unknown_event() {
        let doc = document("<Button label=\"Save\" on:clik=submit/>");
        let result = Compiler::new("app.okui")
            .with_registry(button_registry())
            .compile(&doc);
        assert!(result.has_errors());
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown event 'clik' on component 'Button'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn missing_required_prop_is_reported() {
        let doc = document("<Button on:click=submit/>");
        let result = Compiler::new("app.okui")
            .with_registry(button_registry())
            .compile(&doc);
        assert!(result.diagnostics().iter().any(|d| matches!(
            d,
            CompileDiagnostic::Validation(v)
                if v.message() == "component 'Button' requires prop 'label'"
        )));
    }

    #[test]
    fn diagnostics_from_multiple_passes_accumulate() {
        // An unknown reference (resolution) AND an unknown attribute (contract)
        // are reported together, not one-at-a-time.
        let doc = document("<Button label=\"Save\" bogus={missing}/>");
        let result = Compiler::new("app.okui")
            .with_registry(button_registry())
            .compile(&doc);
        let has_resolution = result
            .diagnostics()
            .iter()
            .any(|d| matches!(d, CompileDiagnostic::Resolution(_)));
        let has_unknown_attr = result.diagnostics().iter().any(|d| {
            matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown attribute 'bogus' on component 'Button'"
            )
        });
        assert!(has_resolution, "diagnostics: {:?}", result.diagnostics());
        assert!(has_unknown_attr, "diagnostics: {:?}", result.diagnostics());
        assert!(result.diagnostics().len() >= 2);
    }

    #[test]
    fn seeded_symbol_resolves_dynamic_attribute() {
        let registry =
            ContractRegistry::new().with(ComponentContract::new("Text").allowing_prop("value"));
        let doc = document("<Text value={count}/>");
        let result = Compiler::new("app.okui")
            .with_registry(registry)
            .defining(Symbol::prop("count"))
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        assert!(result.document().is_some());
    }

    #[test]
    fn granted_capability_satisfies_requirement() {
        let doc = document("<Cam requires=\"camera\"/>");
        let result = Compiler::new("app.okui")
            .granting(CapabilityId::new("camera"))
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn empty_document_yields_no_document_but_a_lowering_diagnostic() {
        let doc = document("");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(result.document().is_none());
        assert!(result.has_errors());
        assert!(
            result
                .diagnostics()
                .iter()
                .any(|d| matches!(d, CompileDiagnostic::Lowering(LoweringError::EmptyDocument)))
        );
    }

    #[test]
    fn for_loop_without_key_surfaces_a_lowering_diagnostic_but_still_lowers() {
        // A keyless <For> is a §2.11 violation reported through the pipeline,
        // yet the document is still produced (the diagnostic is non-fatal).
        let doc = document("<For each={items}><Row/></For>");
        let result = Compiler::new("app.okui")
            .defining(Symbol::prop("items"))
            .compile(&doc);
        assert!(result.document().is_some());
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Lowering(LoweringError::MissingLoopKey(_))
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn virtual_for_compiles_clean_and_lowers_to_a_virtual_collection() {
        // §2-D: a `<VirtualFor>` used to be refused because it emitted the same
        // loop a `<For>` does and kept no windowing promise. It is now a
        // first-class §2.11 construct: it compiles clean and lowers to a virtual
        // collection — NOT to a `<For>` — that the runtime realizes into a
        // viewport-bounded window (spec §4.9).
        let doc = document("<VirtualFor each={items} key={id}><Row/></VirtualFor>");
        let result = Compiler::new("app.okui")
            .defining(Symbol::prop("items"))
            .defining(Symbol::prop("id"))
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        let document = result.document().expect("a lowered document");
        // The IR keeps the construct as a `VirtualFor` loop node...
        assert_eq!(document.root().tag(), "VirtualFor");
        // ...and bridging it produces a virtual-collection marker, not the spliced
        // template a `<For>` bridges to.
        let view = crate::bridge::UiIrViewBridge::document_to_view(document);
        assert!(
            view.root()
                .nodes()
                .iter()
                .any(obscur_view::view::is_virtual_collection),
            "a `<VirtualFor>` bridges to a virtual-collection marker"
        );
    }

    #[test]
    fn a_malformed_outlet_makes_the_compile_report_errors() {
        // §2.12: `has_errors()` is the gate the one production consumer of this
        // pipeline reads (`obscur-kit`'s build.rs panics on it), so the outlet
        // rule is only real if it lands there — the same standard `<VirtualFor>`
        // is held to.
        for (source, expected) in [
            ("<div><slot/></div>", "requires a `name`"),
            ("<div><slot name={which}/></div>", "`<slot name={…}>`"),
            (r#"<slot name="h" class="x"/>"#, "'class'"),
            (r#"<slot name="h"><b>f</b></slot>"#, "takes no children"),
        ] {
            let result = Compiler::new("app.okui")
                .defining(Symbol::prop("which"))
                .compile(&document(source));
            assert!(result.has_errors(), "`{source}` must not compile clean");
            assert!(
                result
                    .diagnostics()
                    .iter()
                    .any(|d| matches!(d, CompileDiagnostic::Validation(v)
                        if v.message().contains(expected))),
                "`{source}` diagnostics: {:?}",
                result.diagnostics()
            );
        }
    }

    #[test]
    fn a_well_formed_outlet_compiles_clean() {
        // The disjoint half: the outlet rule must refuse a malformed `<slot>` and
        // nothing else. A named outlet is what the spelling is *for*, and it also
        // must not trip the contract registry — `slot` is a reserved tag with no
        // contract, so `check_contract` leaves it alone and `check_outlet` owns it.
        let result = Compiler::new("app.okui").compile(&document(
            r#"<div class="region"><slot name="header"/></div>"#,
        ));
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        assert!(result.document().is_some());
    }

    #[test]
    fn a_keyed_for_compiles_clean() {
        // The disjoint half: the collection rule must refuse `<VirtualFor>` and
        // nothing else. A `<For>` with everything §2.11 demands reports nothing.
        let doc = document("<For each={items} key={id}><Row/></For>");
        let result = Compiler::new("app.okui")
            .defining(Symbol::prop("items"))
            .defining(Symbol::prop("id"))
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn compile_diagnostic_display_delegates() {
        let diag = CompileDiagnostic::Lowering(LoweringError::EmptyDocument);
        assert_eq!(diag.to_string(), "cannot lower an empty document");
    }

    // --- audit V5: the default compile path enforces the built-in contracts ---

    #[test]
    fn default_registry_is_populated_with_builtins() {
        // A default compiler carries a non-empty registry; the empty parallel
        // registry that made §2.13 inert is gone.
        let registry = ContractRegistry::builtins();
        assert!(!registry.is_empty());
        assert!(registry.get("Image").is_some());
        assert!(registry.get("Canvas").is_some());
        assert_eq!(Compiler::new("app.okui").registry.len(), registry.len());
    }

    #[test]
    fn default_compile_rejects_unknown_attribute_on_builtin() {
        // No `with_registry`: the built-in `Image` contract is consulted by
        // default and its unknown attribute fails a real compile.
        let doc = document("<Image src=\"logo.png\" widht=\"10\"/>");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown attribute 'widht' on component 'Image'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn default_compile_reports_missing_required_prop_on_builtin() {
        let doc = document("<Image alt=\"logo\"/>");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "component 'Image' requires prop 'src'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn default_compile_accepts_valid_builtin_usage() {
        let doc = document("<Image src=\"logo.png\" alt=\"logo\"/>");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        assert!(result.document().is_some());
    }

    // --- audit V5: user components registered via a ComponentSurface (as
    // `#[derive(Contract)]` emits) are enforced by a real compile, exactly like
    // the built-ins. ---

    fn tabs_registry() -> ContractRegistry {
        // Mirrors what `MyProps::contract_surface()` yields for a `Tabs`
        // component with a required `selected` prop, an optional `orientation`
        // prop, a `change` event and `Tab`-only children.
        let surface = ComponentSurface::new("Tabs")
            .requiring_prop("selected")
            .allowing_prop("orientation")
            .allowing_event("change")
            .allowing_child("Tab");
        ContractRegistry::builtins().with_surface(&surface)
    }

    #[test]
    fn user_surface_rejects_bogus_attribute() {
        let doc = document("<Tabs selected=\"0\" bogus=\"x\"><Tab/></Tabs>");
        let result = Compiler::new("app.okui")
            .with_registry(tabs_registry())
            .compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown attribute 'bogus' on component 'Tabs'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn user_surface_reports_missing_required_prop() {
        let doc = document("<Tabs><Tab/></Tabs>");
        let result = Compiler::new("app.okui")
            .with_registry(tabs_registry())
            .compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "component 'Tabs' requires prop 'selected'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn user_surface_rejects_disallowed_child() {
        let doc = document("<Tabs selected=\"0\"><Panel/></Tabs>");
        let result = Compiler::new("app.okui")
            .with_registry(tabs_registry())
            .compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "'Panel' is not an allowed child of 'Tabs'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn user_surface_rejects_unknown_event() {
        let doc = document("<Tabs selected=\"0\" on:hover=go><Tab/></Tabs>");
        let result = Compiler::new("app.okui")
            .with_registry(tabs_registry())
            .defining(Symbol::prop("go"))
            .compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message() == "unknown event 'hover' on component 'Tabs'"
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn user_surface_accepts_fully_correct_usage() {
        let doc =
            document("<Tabs selected=\"0\" orientation=\"vertical\" on:change=go><Tab/></Tabs>");
        let result = Compiler::new("app.okui")
            .with_registry(tabs_registry())
            .defining(Symbol::prop("go"))
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
        assert!(result.document().is_some());
    }

    // --- audit 2-C: a handler reference is checked against the declared commands ---

    #[test]
    fn a_declared_command_surface_catches_a_misspelled_handler() {
        // The silent no-op, made loud: `on:click=incrment` used to compile clean
        // and produce a button that does nothing, because the name was hashed and
        // matched against whatever the application happened to register.
        let doc = document("<button class=\"btn\" on:click=incrment/>");
        let result = Compiler::new("app.okui")
            .claiming_command("increment")
            .compile(&doc);
        assert!(
            result.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message().contains("unknown command 'incrment'")
                        && v.message().contains("did you mean 'increment'?")
            )),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn the_declared_command_itself_compiles_clean() {
        // The disjoint half: the check must refuse the typo and nothing else.
        let doc = document("<button class=\"btn\" on:click=increment/>");
        let result = Compiler::new("app.okui")
            .claiming_command("increment")
            .compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    #[test]
    fn a_compiler_told_nothing_about_commands_judges_nothing() {
        // The default has to stay silent: a false "unknown command" is worse than
        // the no-op, and every document written before this check existed still
        // compiles.
        let doc = document("<button on:click=whatever_nobody_declared/>");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }

    // --- audit 2-E: a contract can say "no children" ---

    #[test]
    fn the_default_registry_refuses_children_under_a_widget_that_synthesises_them() {
        // `<Button>OK</Button>` is meaningless — the widget builds its label from
        // its own props — and it used to compile, because an empty
        // allowed-children list meant "any child".
        let text = Compiler::new("app.okui").compile(&document("<Button label=\"x\">OK</Button>"));
        assert!(
            text.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v) if v.message().contains("takes no children")
            )),
            "diagnostics: {:?}",
            text.diagnostics()
        );

        let element = Compiler::new("app.okui").compile(&document("<Tabs><Tab/></Tabs>"));
        assert!(
            element.diagnostics().iter().any(|d| matches!(
                d,
                CompileDiagnostic::Validation(v)
                    if v.message().contains("takes no children") && v.message().contains("'Tab'")
            )),
            "an invented child vocabulary is named, not silently mounted as nothing: {:?}",
            element.diagnostics()
        );
    }

    #[test]
    fn the_base_element_spelling_keeps_its_children() {
        // The line that matters: `<button>` is a base element whose authored
        // children are the only thing that renders, and it shares the widget's
        // `ComponentId` — but not its contract, which is keyed by the authored tag.
        // Forbidding children on `<Button/>` must not touch it.
        let result = Compiler::new("app.okui").compile(&document(
            "<button class=\"btn\"><span>press</span></button>",
        ));
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );

        // …and `Dialog`, the one widget whose children ARE its content.
        let dialog =
            Compiler::new("app.okui").compile(&document("<Dialog title=\"Hi\"><p/></Dialog>"));
        assert!(
            !dialog.has_errors(),
            "diagnostics: {:?}",
            dialog.diagnostics()
        );
    }

    #[test]
    fn default_compile_leaves_unknown_tags_permissive() {
        // A tag with no built-in contract stays unchecked (existing policy).
        let doc = document("<CustomThing whatever=\"1\"/>");
        let result = Compiler::new("app.okui").compile(&doc);
        assert!(
            !result.has_errors(),
            "diagnostics: {:?}",
            result.diagnostics()
        );
    }
}
