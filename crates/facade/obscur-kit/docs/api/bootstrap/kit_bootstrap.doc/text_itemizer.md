Builds the real
[`UnicodeTextItemizer`](crate::text_unicode::UnicodeTextItemizer) — the
[`TextItemizer`](crate::text::shaping::TextItemizer) that splits text into
uniform bidi-level + script runs before per-item resolution and shaping.

Install it with
[`set_text_itemizer`](crate::runtime::UiRuntime::set_text_itemizer). The
itemizer is returned boxed as the domain port, so callers depend only on
the port, not the concrete adapter.
