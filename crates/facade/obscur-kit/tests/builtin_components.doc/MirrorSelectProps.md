A props struct declared only so `#[derive(Contract)]` can be run on the same
name the shipped `SelectProps` states by hand, and the two contracts compared.
It is never constructed and never read: the derive's output is the whole
subject.

`options` is an `Option<String>` here and a `Vec<Box<str>>` there, and that is
not drift: a contract records the type of the **authored** value, and a
`ViewValue` has no list — what is on the node is a delimited string. The other
two fields have the same types on both sides.
