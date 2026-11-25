What the style phase left for the layout classifier to compare against
(spec §3.4 phase 5, §4.3).

[`LayoutClassifier::classify`](super::LayoutClassifier::classify) answers "could any box have moved?" by diffing
each node's layout-affecting properties against what it had before. Which nodes
it must ask about depends on how the styles were produced, and so does where
the "before" lives — hence this enum rather than an unconditional whole-map
diff.
