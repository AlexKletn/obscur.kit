#![doc = include_str!("controlled_reseed.doc/_module.md")]

use obscur_tree::tree::RetainedTree;

use super::{EditableField, TextEditSession};

#[doc = include_str!("controlled_reseed.doc/ReseedDecision.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReseedDecision {
    #[doc = include_str!("controlled_reseed.doc/Keep.md")]
    Keep,
    #[doc = include_str!("controlled_reseed.doc/Mirror.md")]
    Mirror(String),
    #[doc = include_str!("controlled_reseed.doc/Reseed.md")]
    Reseed(String),
}

#[doc = include_str!("controlled_reseed.doc/ControlledReseed.md")]
pub(crate) struct ControlledReseed;

impl ControlledReseed {
    #[doc = include_str!("controlled_reseed.doc/decide.md")]
    pub(crate) fn decide(tree: &RetainedTree, session: &TextEditSession) -> ReseedDecision {
        let field = session.node();
        // The field's value as the freshly-reconciled tree now carries it (the
        // placeholder overlay is skipped, exactly as at seed time).
        let retained = EditableField::collect_text(tree, field);
        let declared = EditableField::declared_control_value(tree, field);
        Self::discriminate(retained, declared, session.mirror(), session.text())
    }

    #[doc = include_str!("controlled_reseed.doc/discriminate.md")]
    fn discriminate(
        retained: String,
        declared: Option<String>,
        mirror: &str,
        live: &str,
    ) -> ReseedDecision {
        // (1) Unchanged retained value: a static/uncontrolled field, or a frame
        //     that rewrote something else.
        //
        //     A field that declares an EXPLICIT controlled value (the reserved
        //     `VALUE` attribute the `Input` widget emits — WHATWG HTML §4.10.5.5
        //     value reflection, §3.1 "values flow down") reflects that value: if the
        //     buffer has drifted from a value the app is holding CONSTANT, that
        //     drift is a keystroke the app proposed and refused, so snap the buffer
        //     back — a controlled field must never drift. A field that declares NO
        //     explicit value (a bare editable whose text is its only value) is its
        //     OWN source of truth, so an in-flight edit is kept untouched, never
        //     fought (this is what lets a lag frame paint the session's typed text
        //     while the retained tree still carries the pre-keystroke value).
        if retained == mirror {
            return match declared {
                Some(declared) if declared != live => ReseedDecision::Reseed(declared),
                _ => ReseedDecision::Keep,
            };
        }
        // (2) The session's OWN edit reached the tree (passthrough controlled
        //     input): advance the watermark but keep the live buffer + caret.
        if retained == live {
            return ReseedDecision::Mirror(retained);
        }
        // External rewrite (Clear / normalize): reseed so the field shows the model
        // value, caret preserved (clamped).
        ReseedDecision::Reseed(retained)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn an_unchanged_retained_value_with_no_declared_value_keeps_the_session() {
        assert_eq!(
            ControlledReseed::discriminate("old".into(), None, "old", "the user is typing"),
            ReseedDecision::Keep
        );
    }

    #[test]
    fn a_constant_declared_value_the_buffer_drifted_from_snaps_it_back() {
        // The app is holding `value` constant while the buffer moved: the keystroke
        // was proposed and refused, so a controlled field must not keep the drift.
        assert_eq!(
            ControlledReseed::discriminate("ab".into(), Some("ab".into()), "ab", "abc"),
            ReseedDecision::Reseed("ab".into())
        );
        // …and a buffer that already agrees with the declared value is left alone,
        // so a controlled field costs nothing per frame once it has settled.
        assert_eq!(
            ControlledReseed::discriminate("ab".into(), Some("ab".into()), "ab", "ab"),
            ReseedDecision::Keep
        );
    }

    #[test]
    fn the_sessions_own_edit_coming_back_only_advances_the_watermark() {
        assert_eq!(
            ControlledReseed::discriminate("abc".into(), Some("abc".into()), "ab", "abc"),
            ReseedDecision::Mirror("abc".into()),
            "a passthrough controlled input never disturbs the live buffer"
        );
    }

    #[test]
    fn a_normalising_write_back_reseeds_the_buffer() {
        assert_eq!(
            ControlledReseed::discriminate("ABC".into(), Some("ABC".into()), "AB", "abc"),
            ReseedDecision::Reseed("ABC".into()),
            "an uppercasing on-input differs from the buffer, so the model wins"
        );
    }

    #[test]
    fn a_field_outside_the_tree_reads_as_an_empty_value() {
        // The whole read path over a tree that holds nothing: an empty retained
        // value against an empty freshly-seeded session is `Keep`, so a session on
        // a node the frame dropped is never reseeded out from under itself.
        let tree = RetainedTree::new();
        let session = TextEditSession::seeded(NodeId::from_raw(7), "");
        assert_eq!(
            ControlledReseed::decide(&tree, &session),
            ReseedDecision::Keep
        );
    }
}
