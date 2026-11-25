Appends the allowed-children violations (spec §2.13).

The three [`ChildPolicy`] states are three different verdicts, which is why
the policy is an enum rather than a list whose emptiness had to mean two
things:

- [`Any`](ChildPolicy::Any) — nothing to check;
- [`Only`](ChildPolicy::Only) — each member's tag must be listed; content is
  left alone, since a list of tags says nothing about a text run;
- [`None`](ChildPolicy::None) — every member *and* any content is refused,
  because the component's own `view()` builds its children and an authored one
  reaches no tree.

A member's [`MemberArity`] is deliberately not consulted, exactly as it is not
for [`DisallowedChild`](ContractViolation::DisallowedChild). Arity exists so a
*count* is never asserted beyond what is certain (see
[`check_cardinality`]); a forbidden child is not a count. `<Button><If
cond={x}><span/></If></Button>` may contribute zero nodes at runtime, and is
still markup whose only possible effect is nothing.
