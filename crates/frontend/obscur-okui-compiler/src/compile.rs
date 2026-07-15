//! The top-level `.okui` compilation orchestrator: sequences resolution,
//! validation and lowering over one parsed document into UI IR (spec §2.9–§2.15).
//!
//! The individual passes ([`NameResolver`](crate::resolution::NameResolver),
//! the validators in [`crate::validation`], [`UiIrLowering`]) remain usable
//! standalone; [`Compiler`] merely runs them in order, accumulating every
//! pass's diagnostics into one list rather than bailing on the first failure.

use std::collections::HashMap;

use obscur_capability::CapabilityId;
use obscur_okui_syntax::ast::{Document, Element};
use obscur_ui_ir::document::UiIrDocument;
use obscur_ui_ir::source::SourceMap;
use obscur_view::contract::ComponentSurface;

use crate::lowering::{LoweringError, UiIrLowering};
use crate::resolution::{NameResolver, ResolutionError};
use crate::symbol::{Symbol, SymbolTable};
use crate::typing::TypeError;
use crate::validation::{
    CapabilityValidator, ComponentContract, ContractValidator, ExpressionValidator,
    ValidationDiagnostic, builtin_contracts, widget_contracts,
};

/// A lookup from component/tag name to its [`ComponentContract`], so the
/// [`ContractValidator`] can find the allowed surface of each element instance
/// (spec §2.13).
///
/// A contract is keyed by its [`component`](ComponentContract::component) name.
/// An element whose tag has no registered contract is left unchecked (the
/// registry does not constrain it).
#[derive(Debug, Clone, Default)]
pub struct ContractRegistry {
    contracts: HashMap<Box<str>, ComponentContract>,
}

impl ContractRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry pre-populated with the contracts for every built-in
    /// view primitive **and every shipped widget** (spec §2.6, §2.13), so a
    /// default compile enforces their attribute/event surface without the caller
    /// registering anything.
    ///
    /// This is the registry [`Compiler::new`] installs by default. See
    /// [`builtin_contracts`](crate::validation::builtin_contracts) and
    /// [`widget_contracts`](crate::validation::widget_contracts) for what each
    /// covers, and why the surfaces are defined here rather than imported from
    /// `obscur-view`/`obscur-primitives`/`obscur-widgets`.
    #[must_use]
    pub fn builtins() -> Self {
        let mut registry = Self::new();
        for contract in builtin_contracts().into_iter().chain(widget_contracts()) {
            registry.register(contract);
        }
        registry
    }

    /// Registers `contract` (keyed by its component name) and returns the
    /// registry (fluent).
    #[must_use]
    pub fn with(mut self, contract: ComponentContract) -> Self {
        self.register(contract);
        self
    }

    /// Registers `contract`, returning the contract it displaced for the same
    /// component name, if any.
    pub fn register(&mut self, contract: ComponentContract) -> Option<ComponentContract> {
        self.contracts.insert(contract.component().into(), contract)
    }

    /// Registers a user component's string [`ComponentSurface`] (bridged into a
    /// string [`ComponentContract`]), so the `.okui` compiler enforces §2.13 for
    /// that component just as it does for the built-ins.
    ///
    /// A real build wires user components in explicitly, e.g.
    /// `registry.register_surface(&MyProps::contract_surface())`, where
    /// `contract_surface()` is emitted by `#[derive(Contract)]`. Automatic,
    /// inventory-style collection of every derived surface is a possible future
    /// follow-up; there is no auto-discovery today.
    ///
    /// Returns the contract it displaced for the same component name, if any.
    pub fn register_surface(&mut self, surface: &ComponentSurface) -> Option<ComponentContract> {
        self.register(surface.into())
    }

    /// Registers a user component's [`ComponentSurface`] and returns the
    /// registry (fluent); see [`register_surface`](Self::register_surface).
    #[must_use]
    pub fn with_surface(mut self, surface: &ComponentSurface) -> Self {
        self.register_surface(surface);
        self
    }

    /// Returns the contract registered for `component`, if any.
    #[must_use]
    pub fn get(&self, component: &str) -> Option<&ComponentContract> {
        self.contracts.get(component)
    }

    /// Returns the number of registered contracts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    /// Returns whether no contracts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }
}

/// A single diagnostic produced by the compilation pipeline: the sum of every
/// pass's error type, so callers see one uniform stream (spec §2.9–§2.15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileDiagnostic {
    /// Name resolution failed for a reference or declaration (§2.10, §2.11).
    Resolution(ResolutionError),
    /// A type check failed (§2.9). Reserved for the future whole-document type
    /// pass; the orchestrator does not emit it yet (see [`Compiler::compile`]).
    Type(TypeError),
    /// A static validation rule was violated — an expression, capability or
    /// contract check (§2.10, §2.13).
    Validation(ValidationDiagnostic),
    /// Lowering to UI IR failed (§2.15).
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

/// The outcome of compiling a `.okui` document (spec §2.15).
///
/// # Policy
///
/// Every pass's diagnostics are accumulated (the pipeline never bails on the
/// first error), so `diagnostics` may hold several independent failures. The
/// `document` is `Some` when lowering could produce UI IR and `None` when a hard
/// lowering error (an empty document or a tagless element) prevented it — even
/// then, all earlier diagnostics are still reported. `source_map` is the lowered
/// document's map, or empty when no document was produced.
#[derive(Debug, Clone)]
pub struct CompileResult {
    document: Option<UiIrDocument>,
    diagnostics: Vec<CompileDiagnostic>,
    source_map: SourceMap,
}

impl CompileResult {
    /// Returns the lowered UI IR document, if one was produced.
    #[must_use]
    pub fn document(&self) -> Option<&UiIrDocument> {
        self.document.as_ref()
    }

    /// Returns the accumulated diagnostics from every pass.
    #[must_use]
    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }

    /// Returns the source map linking IR node ids back to source spans.
    #[must_use]
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Returns whether any pass reported a diagnostic.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Consumes the result, returning just the lowered document, if any.
    #[must_use]
    pub fn into_document(self) -> Option<UiIrDocument> {
        self.document
    }
}

/// Runs the full `.okui` compilation pipeline over one parsed document,
/// producing UI IR and a unified diagnostic stream (spec §2.9–§2.15).
///
/// The compiler is configured fluently with the component contracts
/// ([`with_registry`](Self::with_registry)), the granted capabilities
/// ([`granting`](Self::granting)) and the props/state symbols in scope
/// ([`defining`](Self::defining)) before [`compile`](Self::compile) is called.
#[derive(Debug, Clone, Default)]
pub struct Compiler {
    source_name: Box<str>,
    registry: ContractRegistry,
    granted: Vec<CapabilityId>,
    symbols: Vec<Symbol>,
}

impl Compiler {
    /// Creates a compiler whose lowered source map records spans under
    /// `source_name` (the logical `.okui` source identifier).
    ///
    /// The contract registry is seeded with the built-in primitive contracts
    /// ([`ContractRegistry::builtins`]) so a default compile already rejects
    /// unknown attributes/events on the built-in primitives (spec §2.13);
    /// [`with_registry`](Self::with_registry) replaces it wholesale.
    #[must_use]
    pub fn new(source_name: impl Into<Box<str>>) -> Self {
        Self {
            source_name: source_name.into(),
            registry: ContractRegistry::builtins(),
            ..Self::default()
        }
    }

    /// Sets the contract registry the [`ContractValidator`] consults (fluent).
    #[must_use]
    pub fn with_registry(mut self, registry: ContractRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Grants `capability` to the document being compiled (fluent), so a
    /// matching `requires="…"` declaration is satisfied.
    #[must_use]
    pub fn granting(mut self, capability: CapabilityId) -> Self {
        self.granted.push(capability);
        self
    }

    /// Defines `symbol` (a prop or state binding) in the document's root scope so
    /// its expressions may reference it (fluent).
    #[must_use]
    pub fn defining(mut self, symbol: Symbol) -> Self {
        self.symbols.push(symbol);
        self
    }

    /// Compiles `document`, running each pass in order and accumulating every
    /// diagnostic into the returned [`CompileResult`] (spec §2.15).
    ///
    /// # Pipeline
    ///
    /// 1. **Name resolution** — every reference is resolved against a symbol
    ///    table seeded with the [`defining`](Self::defining) symbols.
    /// 2. **Type checking** — no whole-document pass exists yet: the type layer
    ///    is a per-expression vocabulary/algebra pending an expression AST
    ///    (§2.9/§2.10), so nothing is emitted here. The
    ///    [`Type`](CompileDiagnostic::Type) variant reserves its place in the
    ///    diagnostic sum for when that pass lands.
    /// 3. **Validation** — expression restrictions (§2.10), capability grants
    ///    (§2.13) and, for every element whose tag has a registered contract,
    ///    the contract surface (required props, unknown attributes/events,
    ///    allowed children — §2.13).
    /// 4. **Lowering** — the document is lowered to a [`UiIrDocument`]; on a hard
    ///    lowering error the document is `None` but all earlier diagnostics are
    ///    still returned.
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

        // 3. Validation: expressions, capabilities, then contracts.
        for diagnostic in ExpressionValidator::new().validate_document(document) {
            diagnostics.push(CompileDiagnostic::Validation(diagnostic));
        }
        let capability_validator = CapabilityValidator::new(self.granted.iter().cloned());
        for diagnostic in capability_validator.validate(document) {
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

    /// Validates `element` (and its element-shaped descendants) against any
    /// contract registered for its tag, recursively.
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
