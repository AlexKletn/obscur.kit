`element := '<' Ident attribute* ('/>' | '>' child* close)`

Precondition: `peek() == LAngle` and the next token is an `Ident`.
