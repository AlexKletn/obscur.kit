A container query is a conjunction of its features.

The container name is ignored: nothing can establish a *named* container any
more than an anonymous one, so matching the name would not change the answer.
A query with no features at all (`@container name { … }`) asks only "does
this container exist", which is [`Never`](RuleCondition::Never) here.
