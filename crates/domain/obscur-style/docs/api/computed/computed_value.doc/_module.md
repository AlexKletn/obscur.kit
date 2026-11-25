A single, fully-resolved computed value: [`ComputedValue`].

A [`ComputedValue`] is the end product of computed-value resolution (§4.14)
for one property: it is guaranteed **concrete**. Concrete means it is never a
CSS-wide keyword ([`PropertyValue::Inherit`], [`PropertyValue::Initial`],
[`PropertyValue::Unset`]) and never an unresolved custom-property reference
([`PropertyValue::CustomRef`]) — those placeholders are all eliminated during
resolution. Only concrete leaf values (keyword / length / number / color)
remain.
