//! The §2.11 collection rule: which of the two collection constructs this
//! pipeline will compile, stated over a plain tag string so **both** front ends
//! decide it the same way.
//!
//! # Why `<VirtualFor>` is refused
//!
//! §2.11 names two collection constructs, `<For>` and `<VirtualFor>`. They are
//! not two spellings of one thing: a `<VirtualFor>` is a *performance contract*.
//! §4.9 says what it promises — materialization of a window rather than the whole
//! collection, overscan around it, estimated geometry for what is not measured,
//! and scroll anchoring across changes.
//!
//! Nothing here keeps that promise. Both tags lower to
//! [`UiIrNodeKind::Loop`](obscur_ui_ir::document::UiIrNodeKind::Loop), and
//! `ViewCodegen::emit_loop` never reads the tag: it emits one `for item in
//! items.iter()`, which materializes every item, for either. A `<VirtualFor>`
//! over a million rows built a million `ViewNode`s and said nothing about it.
//!
//! So the tag is refused, the way an unsupported CSS feature is refused rather
//! than approximated: an author asking for windowed rendering is told they are
//! not getting it, instead of shipping a `<For>` under another name and finding
//! out at a million rows. `<For>` remains, and is exactly what a `<VirtualFor>`
//! compiled to anyway.
//!
//! # What it would take to accept it
//!
//! Not a `match` arm here. `visible_range(scroll_offset, viewport_extent,
//! item_count)` — the signature of the mechanism that already exists,
//! `obscur_layout::virtualization::VirtualLayout` — names the two inputs the
//! emitted code has no way to reach: a *view builder* runs before layout and is
//! handed no scroll offset and no viewport. `document_to_view_fn` emits a nullary
//! `pub fn <name>() -> View`, so a compiled file's loop cannot read either from
//! anywhere at all, and §4.9's estimated geometry and scroll anchoring are
//! retained state across frames that a `for` loop in a view builder does not
//! have. That is a retained collection consumer, not a codegen change; when one
//! exists, this rule is the one place that says no.
//!
//! # One rule, two front ends
//!
//! The rule takes a `&str` tag and returns a message, with no syntax in it, for
//! the same reason [`check_expression`](crate::validation::check_expression) and
//! [`check_contract`](crate::validation::check_contract) do: there are two
//! authoring surfaces and only one of them has a `.okui` syntax tree. Each front
//! end attaches its own location — a byte offset for the file path, a `Span` for
//! the `okui!` macro. A rule written against either input can only ever check
//! that input, and `<VirtualFor>` is in this file *because* the two front ends
//! agreed to ignore it together.

/// The diagnostic reported for a `<VirtualFor>` (spec §2.11, §4.9).
///
/// A single constant so both front ends report the identical text, and a test can
/// name what it expects without matching on prose.
pub const VIRTUAL_FOR_MESSAGE: &str = "`<VirtualFor>` is not implemented: its virtualization contract (spec §4.9 — materialization, overscan, estimated geometry, scroll anchoring) is honoured by nothing, and it compiles to the same loop as `<For>`, which materializes every item. Use `<For>`";

/// Rejects a §2.11 collection tag this pipeline cannot honour, returning why.
///
/// This is *the* §2.11 collection check. `Ok(())` for every tag that is not a
/// collection construct, and for `<For>` — the construct that is really
/// implemented. `Err(VIRTUAL_FOR_MESSAGE)` for `<VirtualFor>`; see the module
/// docs for why refusing beats accepting.
///
/// # Errors
///
/// Returns the diagnostic message for a collection tag whose contract this
/// pipeline does not keep.
pub fn check_collection(tag: &str) -> Result<(), &'static str> {
    if tag == "VirtualFor" {
        return Err(VIRTUAL_FOR_MESSAGE);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{VIRTUAL_FOR_MESSAGE, check_collection};

    #[test]
    fn plain_for_is_accepted() {
        assert_eq!(check_collection("For"), Ok(()));
    }

    #[test]
    fn virtual_for_is_refused_with_the_shared_message() {
        assert_eq!(check_collection("VirtualFor"), Err(VIRTUAL_FOR_MESSAGE));
    }

    #[test]
    fn an_ordinary_tag_is_not_this_rules_business() {
        for tag in ["div", "Button", "If", "Else", "Empty", "Separator"] {
            assert_eq!(check_collection(tag), Ok(()), "{tag}");
        }
    }

    #[test]
    fn the_message_names_the_construct_and_the_replacement() {
        // The author has to learn two things from one line: that the tag they
        // wrote is not honoured, and what to write instead.
        assert!(VIRTUAL_FOR_MESSAGE.contains("VirtualFor"));
        assert!(VIRTUAL_FOR_MESSAGE.contains("`<For>`"));
    }
}
