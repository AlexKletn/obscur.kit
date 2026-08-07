Lowers a media feature (viewport width/height axes), returning [`None`] for a
feature this pipeline cannot represent.

The caller — not this function — decides what an unrepresentable feature
means, because the decision belongs to the whole query: one such feature sinks
the query to [`RuleCondition::Never`].
