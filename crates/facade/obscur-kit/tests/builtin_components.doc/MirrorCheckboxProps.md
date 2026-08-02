A props struct declared only so `#[derive(Contract)]` can be run on the
same name the shipped `CheckboxProps` states by hand, and the two
contracts compared. It is never constructed and never read: the derive's
output is the whole subject.
