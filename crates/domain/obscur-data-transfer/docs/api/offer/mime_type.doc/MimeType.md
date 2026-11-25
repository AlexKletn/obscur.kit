A MIME type descriptor identifying the format of an offered representation
(spec §6.20).

A data offer advertises its content under one or more MIME types — plain
text, rich text, a URI list, an image, or a custom/opaque byte format. This
value is the pure descriptor: its `essence` (the `type/subtype` string, e.g.
`text/plain`) is normalized to lowercase on construction so that two
descriptors compare equal regardless of the casing a host reported. Common
formats are provided as named constructors ([`MimeType::text_plain`] and
friends) so callers do not re-spell the canonical strings.
