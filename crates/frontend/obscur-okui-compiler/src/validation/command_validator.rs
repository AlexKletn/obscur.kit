//! `CommandValidator`: checks every `on:<event>` handler reference in a document
//! against the commands the caller declared (spec §2.13).

use obscur_okui_syntax::ast::{Document, Element};

use crate::lowering::{attribute_value_text, event_local_name};
use crate::validation::{CommandSurface, ValidationDiagnostic};

/// Checks that every handler a document references is a command somebody claims
/// (spec §2.13, "Unknown attributes, events and **string method names** are
/// compile errors").
///
/// # What it walks, and what it reads
///
/// Every element in the document, recursively, and for each of its `on:<event>`
/// attributes the handler reference: the *same* string the lowering's own
/// `attribute_value_text` hands it, which is the string
/// `name_binding::fallback_symbol` hashes into the command symbol the runtime
/// dispatches on. Reading it through that one function is deliberate — a checker
/// with its own extraction could admit a name the IR does not carry, and a
/// validator that disagrees with the artefact is worse than none.
///
/// All three authored spellings are therefore covered by construction:
/// `on:click=increment`, `on:click={increment}` and (already refused as string
/// dispatch by the [`ContractValidator`](super::ContractValidator))
/// `on:click="increment"`.
///
/// # Only a declared surface can be judged
///
/// The verdict comes from a [`CommandSurface`], and the default
/// [`Unchecked`](CommandSurface::Unchecked) surface reports nothing at all. A
/// command is claimed by a runtime registration, which does not exist while
/// anything is compiling, so the compiler cannot discover the set — the caller
/// declares it, exactly as it declares granted capabilities. Guessing instead
/// would turn a silent no-op into a false compile error, which is the worse of the
/// two failures.
///
/// # The empty-handler case
///
/// `on:click` with no value at all lowers to an empty handler string, which hashes
/// to a symbol nothing can ever register. It is reported as its own diagnostic
/// rather than as an unclaimed name, because "the command '' is not claimed" would
/// name nothing to fix.
#[derive(Debug, Clone, Default)]
pub struct CommandValidator {
    surface: CommandSurface,
}

impl CommandValidator {
    /// Creates a validator judging handler references against `surface`.
    #[must_use]
    pub const fn new(surface: CommandSurface) -> Self {
        Self { surface }
    }

    /// The surface this validator judges against.
    #[must_use]
    pub const fn surface(&self) -> &CommandSurface {
        &self.surface
    }

    /// Validates every element in `document`, recursively, reporting each handler
    /// reference no declared command claims (spec §2.13).
    ///
    /// Returns nothing when the surface is
    /// [`Unchecked`](CommandSurface::Unchecked) — including the empty-handler
    /// diagnostic, which is a statement about the declared surface too: with no
    /// declaration, a document's bindings are simply not this pass's business.
    #[must_use]
    pub fn validate(&self, document: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        if !self.surface.is_checked() {
            return diagnostics;
        }
        for element in document.elements() {
            self.validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    /// Checks one element's event bindings, then recurses into its element-shaped
    /// children.
    fn validate_element(&self, element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        let element_offset = element.syntax().offset().unwrap_or(0);
        for attribute in element.attributes() {
            if !attribute.is_event() {
                continue;
            }
            let event = event_local_name(&attribute);
            let handler = attribute_value_text(&attribute).unwrap_or_default();
            let offset = attribute.syntax().offset().unwrap_or(element_offset);
            let tag = element.tag_name().unwrap_or_default();
            if handler.is_empty() {
                diagnostics.push(ValidationDiagnostic::new(
                    format!(
                        "'on:{event}' on '{tag}' names no handler; a binding with no command \
                         dispatches nothing (spec §2.13)"
                    ),
                    offset,
                ));
                continue;
            }
            if self.surface.claims(&handler) {
                continue;
            }
            let suggestion = self
                .surface
                .nearest(&handler)
                .map_or_else(String::new, |near| format!(" — did you mean '{near}'?"));
            diagnostics.push(ValidationDiagnostic::new(
                format!(
                    "unknown command '{handler}' bound by 'on:{event}' on '{tag}': no handler \
                     claims it, so the binding would dispatch to nothing (spec §2.13){suggestion}"
                ),
                offset,
            ));
        }
        for child in element.children() {
            if let Some(child_element) = Element::cast(child) {
                self.validate_element(&child_element, diagnostics);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    fn messages(surface: CommandSurface, src: &str) -> Vec<String> {
        CommandValidator::new(surface)
            .validate(&document(src))
            .iter()
            .map(|d| d.message().to_owned())
            .collect()
    }

    #[test]
    fn an_undeclared_surface_reports_nothing() {
        assert!(
            messages(
                CommandSurface::Unchecked,
                "<button on:click=whatever_it_is/>"
            )
            .is_empty()
        );
    }

    #[test]
    fn a_claimed_command_passes_in_both_spellings() {
        let surface = CommandSurface::declared(["increment"]);
        assert!(messages(surface.clone(), "<button on:click=increment/>").is_empty());
        assert!(messages(surface, "<button on:click={increment}/>").is_empty());
    }

    #[test]
    fn an_unclaimed_command_is_reported_with_the_handler_named() {
        let reported = messages(
            CommandSurface::declared(["increment"]),
            "<button on:click=incrment/>",
        );
        assert_eq!(reported.len(), 1);
        assert!(
            reported[0].contains("unknown command 'incrment'"),
            "the offending handler is named: {}",
            reported[0]
        );
        assert!(
            reported[0].contains("did you mean 'increment'?"),
            "and the nearest declared name is offered: {}",
            reported[0]
        );
        assert!(
            reported[0].contains("'on:click'") && reported[0].contains("'button'"),
            "with the binding and the element it sits on: {}",
            reported[0]
        );
    }

    #[test]
    fn the_braced_spelling_is_checked_too() {
        // `{increment}` is not an expression to be evaluated — it is the same
        // handler reference, and the lowering takes the same text from it.
        let reported = messages(
            CommandSurface::declared(["increment"]),
            "<button on:click={incrment}/>",
        );
        assert_eq!(reported.len(), 1, "{reported:?}");
        assert!(reported[0].contains("unknown command 'incrment'"));
    }

    #[test]
    fn a_nested_binding_is_checked() {
        let reported = messages(
            CommandSurface::declared(["save"]),
            "<div><span><button on:click=sav/></span></div>",
        );
        assert_eq!(reported.len(), 1, "{reported:?}");
        assert!(reported[0].contains("did you mean 'save'?"));
    }

    #[test]
    fn a_binding_with_no_handler_at_all_is_its_own_diagnostic() {
        // The empty string hashes to a symbol nothing can register; "unknown
        // command ''" would name nothing to fix.
        let reported = messages(CommandSurface::declared(["save"]), "<button on:click/>");
        assert_eq!(reported.len(), 1, "{reported:?}");
        assert!(
            reported[0].contains("names no handler"),
            "got: {}",
            reported[0]
        );
    }

    #[test]
    fn an_empty_declaration_refuses_every_binding() {
        let reported = messages(
            CommandSurface::declared(Vec::<&str>::new()),
            "<button on:click=increment/>",
        );
        assert_eq!(reported.len(), 1, "{reported:?}");
        assert!(reported[0].contains("unknown command 'increment'"));
        assert!(
            !reported[0].contains("did you mean"),
            "there is nothing to suggest: {}",
            reported[0]
        );
    }

    #[test]
    fn a_property_that_merely_looks_like_a_command_is_left_alone() {
        // Only `on:` bindings are handler references; an ordinary attribute holds a
        // value and is the name resolver's business, not this pass's.
        assert!(
            messages(
                CommandSurface::declared(["increment"]),
                "<div class=\"incrment\" title={whatever}/>"
            )
            .is_empty()
        );
    }
}
