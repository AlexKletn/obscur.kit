The result of lowering one [`ParsedDeclaration`].

A declaration lowers to exactly one of these: a standard cascade
[`Declaration`], a [`CustomProperty`], or an [`UnsupportedCssFeature`]
diagnostic when the property name or its value cannot be lowered.
