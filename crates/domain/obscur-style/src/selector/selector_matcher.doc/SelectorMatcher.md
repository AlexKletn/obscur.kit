The stateless engine that tests selectors against a [`MatchElement`].

Matching a complex selector is done **right-to-left**: the subject (rightmost)
compound is tested against the element, then each combinator to its left is
walked against parents or preceding siblings. Descendant and
subsequent-sibling combinators backtrack — they try every candidate
ancestor / preceding sibling and succeed if any lets the remaining prefix
match — while child and next-sibling combinators consider only the single
immediate parent / previous sibling.
