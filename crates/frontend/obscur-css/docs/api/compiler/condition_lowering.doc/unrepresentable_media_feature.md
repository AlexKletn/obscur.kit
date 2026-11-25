Builds the report for a media feature the pipeline cannot evaluate.

It is filed as an [`AtRule`](crate::error::UnsupportedFeatureKind::AtRule)
fault, matching how `@container` is already reported, because the consequence
is at-rule shaped: the condition cannot be answered, so the whole block is
dropped. The parsed condition carries no source offsets, so the span is empty
and the feature name in the message is what locates the fault.
