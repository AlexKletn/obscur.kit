Reads a single authored [`ViewValue`] as a concrete Rust type (spec §3.1).

[`ViewValue`] is the small closed enum a `.okui` document or an `okui!` macro
produces (`Null`/`Bool`/`Int`/`Float`/`Text`). This trait is the per-type half
of prop decoding: [`FromProps`](crate::component::FromProps) finds the value
for a property, and this turns it into the field's type.

# Strict, with one deliberate widening

Decoding does not coerce across kinds: a `Text` never becomes a `bool`, and a
`Bool` never becomes an integer. An authored mistake should surface as a
[`PropsError::Type`](crate::component::PropsError::Type), not as a silently
wrong value.

The single exception is **`Int` → floating point**: an author writing
`opacity={1}` means `1.0`, and requiring `{1.0}` would be a papercut with no
safety benefit. `Float` → integer is *not* accepted (it would have to
truncate), and an `Int` outside the target's range is a
[`Type`](crate::component::PropsError::Type) error rather than a wrap.

# Implementing it

Return [`None`] when the value cannot be read; the caller turns that into the
error, so implementors do not need a property name. [`TYPE_NAME`](Self::TYPE_NAME)
is what the resulting diagnostic reports as "wanted".
