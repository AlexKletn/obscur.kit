The same class list `has_class` searches, handed out whole.

A node the tree no longer holds carries no classes, which is the same answer
`has_class` gives it, so the enumeration succeeds with nothing to visit rather
than declining. Declining would be read as "ask me one at a time", and there
is nothing to ask.
