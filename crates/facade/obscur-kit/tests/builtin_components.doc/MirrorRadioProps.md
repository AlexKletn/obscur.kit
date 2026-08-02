A props struct declared only so `#[derive(Contract)]` can be run on the same
name the shipped `RadioProps` states by hand, and the two contracts compared.
It is never constructed and never read: the derive's output is the whole
subject.

The `#[contract(default, name = "checked")]` is the mirror of the decision
`RadioProps` documents: the field is the widget's `selected`, the *prop* is the
node's reserved `checked`.
