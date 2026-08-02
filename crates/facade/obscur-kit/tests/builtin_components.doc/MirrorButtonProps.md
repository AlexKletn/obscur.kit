A props struct declared only so `#[derive(Contract)]` can be run on the same
name the shipped `ButtonProps` states by hand, and the two contracts
compared. It is never constructed and never read: the derive's output is the
whole subject.

It carries `#[contract(slot(default))]`, so it is compared whole like the other
four. That was not always true: this mirror used to be compared field by field
with the slot excluded, because the hand-written contract declares the §2.12
**default slot** and no derive attribute could name a slot that has no name.
§2.12 now states the default slot and `slot(default)` is its spelling.
