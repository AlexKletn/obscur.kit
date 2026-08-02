A props struct declared only so `#[derive(Contract)]` can be run on the same
name the shipped `ButtonProps` states by hand, and the two contracts
compared. It is never constructed and never read: the derive's output is the
whole subject.

Unlike the other mirrors, this one is compared field by field rather than
whole: the hand-written contract declares the §2.12 **default slot**, and no
derive attribute can name a slot that has no name.
