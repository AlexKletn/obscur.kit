Reserved symbol for a replaced element's image *source* — the opaque key an
`<img>` / [`Image`](../../../obscur_primitives/struct.Image.html) primitive
carries (as [`ViewValue::Text`]). Unlike `class` / `id` / `type`, this is not a
CSS selector input: it is the element's own *content*, resolved by the runtime
to decoded pixels and painted over the element's box. Kept in the same reserved
low-symbol namespace so it survives reconciliation onto the retained node.
