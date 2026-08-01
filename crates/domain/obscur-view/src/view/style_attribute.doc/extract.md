Extracts the well-known style attributes from a node's properties.

Reads the [`CLASS`], [`ID`], [`TYPE`], [`SRC`], [`DISABLED`], [`CHECKED`],
and [`READONLY`] reserved symbols. The string-valued inputs
([`CLASS`]/[`ID`]/[`TYPE`]/[`SRC`]) contribute only from
[`ViewValue::Text`] values (other value kinds are ignored); the `class`
text is split on ASCII whitespace into individual class names, preserving
order and dropping empties, and the [`SRC`] text is carried verbatim. The
boolean state inputs ([`DISABLED`]/[`CHECKED`]/[`READONLY`]) accept any
value kind and are set when it is truthy (bare presence or a non-`false`
value).
