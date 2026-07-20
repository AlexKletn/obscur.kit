//! The §2.11 collection rule: which collection constructs this pipeline
//! compiles, stated over a plain tag string so **both** front ends decide it the
//! same way.
//!
//! # Both constructs are honoured
//!
//! §2.11 names two collection constructs, `<For>` and `<VirtualFor>`, and this
//! pipeline compiles both. They are not two spellings of one thing: a `<For>`
//! materializes one node per item, and a `<VirtualFor>` is a *performance
//! contract* — §4.9 asks it to materialize a **window** of the collection rather
//! than the whole of it, widened by overscan, with the geometry of what is not
//! yet measured estimated.
//!
//! That contract is now kept, so the tag is accepted rather than refused. A
//! `<VirtualFor>` no longer lowers to the same loop a `<For>` does: it lowers to
//! a single [`VIRTUAL_COLLECTION`](obscur_view::view::VIRTUAL_COLLECTION) marker
//! carrying its row template, and the runtime — which owns the viewport —
//! realizes that marker into a viewport-bounded window through
//! [`VirtualLayout`](obscur_layout::virtualization::VirtualLayout). The realized
//! count is therefore a function of the viewport, not of the collection's
//! length, which is the promise a `<For>` cannot make. The bridge
//! ([`UiIrViewBridge`](crate::bridge::UiIrViewBridge)) does this for the direct
//! IR→view path and the codegen ([`ViewCodegen`](crate::codegen::ViewCodegen))
//! emits the same marker for the build-time path.
//!
//! # One rule, two front ends
//!
//! The rule takes a `&str` tag and returns a verdict, with no syntax in it, for
//! the same reason [`check_expression`](crate::validation::check_expression) and
//! [`check_contract`](crate::validation::check_contract) do: there are two
//! authoring surfaces and only one of them has a `.okui` syntax tree. Each front
//! end attaches its own location — a byte offset for the file path, a `Span` for
//! the `okui!` macro. Keeping the rule here, even now that it accepts every
//! §2.11 tag, is the seam a future collection constraint (a malformed
//! construct, an unsupported nesting) would live in without either front end
//! having to grow its own copy.

/// Checks a §2.11 collection tag. Both collection constructs this pipeline knows
/// — `<For>` and `<VirtualFor>` — are honoured, so every tag is accepted.
///
/// `Ok(())` for every tag: `<For>` materializes per item, `<VirtualFor>`
/// realizes a viewport-bounded window (spec §4.9), and no other tag is this
/// rule's business. The seam remains so a future §2.11 constraint has one home
/// both front ends share; see the module docs.
///
/// # Errors
///
/// Currently never returns an error — both §2.11 constructs are compiled. The
/// [`Result`] shape is kept for the front ends that call it and for the future
/// constraint the module docs describe.
pub fn check_collection(_tag: &str) -> Result<(), &'static str> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_collection;

    #[test]
    fn plain_for_is_accepted() {
        assert_eq!(check_collection("For"), Ok(()));
    }

    #[test]
    fn virtual_for_is_now_accepted() {
        // §2.11's `<VirtualFor>` is a first-class construct: it lowers to a
        // virtual-collection marker the runtime realizes into a viewport-bounded
        // window (spec §4.9), so the rule no longer refuses it.
        assert_eq!(check_collection("VirtualFor"), Ok(()));
    }

    #[test]
    fn an_ordinary_tag_is_not_this_rules_business() {
        for tag in ["div", "Button", "If", "Else", "Empty", "Separator"] {
            assert_eq!(check_collection(tag), Ok(()), "{tag}");
        }
    }
}
