The selectable attributes a [`FontProvider`](crate::font::FontProvider) uses
to pick a concrete face within a family (spec §5.18).

A family names *which* typeface; these attributes name *which cut* of it —
its weight, slant, and width. The provider chooses the registered face whose
attributes are nearest the requested ones by [`FontAttributes::distance`].
