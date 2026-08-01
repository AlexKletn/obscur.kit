Which properties the computed-value step visits, and in which role:
`COMPUTED_PROPERTIES` and `FACE_PROPERTIES`.

Two tables over the same catalogue, answering two different questions about
every [`PropertyId`]: *does the resolver compute you at all*, and *do you help
select the face the `0` glyph is measured in*. They live beside the resolver
rather than inside it because neither is resolution — they are the statement
of what one pass covers, and the first is kept exhaustive by a test
(`computed_properties_cover_every_property_id`) that has nothing to say about
how a value is computed.
