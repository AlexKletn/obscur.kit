A screen authoring `<Badge/>` by its tag, with content for its slot and a
plain `<div>` beside it.

The sibling box is not scenery. If a component tag ever fell back to the
generic container primitive, the host would register `Badge` under the
container's id, `<Badge/>` would emit that same id, the two would agree — and
every `<div>` on the screen would mount a badge. Agreement is not identity;
exactly one node here may mount.
