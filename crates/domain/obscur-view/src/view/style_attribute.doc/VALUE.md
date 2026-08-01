Reserved symbol for a form control's **controlled value** — the authoritative
value the application declares for the field (WHATWG HTML §4.10.5.5 *value
reflection*; obscur.kit §3.1 "values flow down"). Like [`SRC`] (and unlike
`class`/`id`/`type`) this is **not** a CSS selector input: it is the field's
own model value, carried verbatim (as [`ViewValue::Text`]). A widget emits it
to declare "this field is controlled — its buffer must reflect this value";
the runtime reads it to snap a focused editable's buffer back when the app
holds the value constant while the user typed (a refused proposal). A bare
editable that emits no `VALUE` is its own source of truth and is never snapped.
Same reserved low-symbol namespace so it survives reconciliation.
