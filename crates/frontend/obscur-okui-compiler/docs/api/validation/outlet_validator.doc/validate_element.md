Checks one element, then recurses into its element-shaped children.

The walk does not stop at a refusal, and it does not stop at an outlet
either: a document with two bad `<slot>`s reports both, the same
report-them-all policy `Compiler::compile` follows.
