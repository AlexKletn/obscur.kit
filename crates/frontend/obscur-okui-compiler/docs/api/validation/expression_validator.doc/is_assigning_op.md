Whether `op` mutates its left operand (`+=`, `-=`, …).

A separate question from "is it a binary operator", because `syn` gives the
compound-assignment operators the same `Expr::Binary` node as `+` and `==`.
