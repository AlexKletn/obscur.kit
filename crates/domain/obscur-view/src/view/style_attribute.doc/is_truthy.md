Whether a reserved boolean state attribute's value counts as *set*. Bare
presence ([`ViewValue::Null`]) or any truthy value is `true`; an explicit
`false` (`Bool(false)` / `Text("false")` / zero / empty text) is `false`.
