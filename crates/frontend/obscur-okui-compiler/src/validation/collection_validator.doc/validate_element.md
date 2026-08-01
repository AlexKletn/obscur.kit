Checks one element's tag, then recurses into its element-shaped children.

The walk does not stop at a diagnostic: it reports every offending
element, the same report-them-all policy `Compiler::compile` follows.
