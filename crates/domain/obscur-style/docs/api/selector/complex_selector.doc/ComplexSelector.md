A complex selector — a lead compound followed by combinator-linked compounds
(`div > .a + span`).

The compounds are stored left-to-right: `head` is the leftmost compound and
each `tail` entry pairs the combinator that *precedes* a compound with that
compound. The **rightmost** compound (the last `tail` entry, or `head` when
`tail` is empty) is the **subject**: the element being tested must match the
subject, while the earlier compounds constrain its ancestors/siblings. This
is why the matcher evaluates right-to-left, starting from the subject.
