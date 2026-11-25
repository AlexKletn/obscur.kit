Creates a request to analyze `text`, located at `range` in the document
and stamped with `version` (spec §5.13).

The request initially requests **all** services (an empty
[`services`](Self::services) list) and has no
[`ignored_ranges`](Self::ignored_ranges); narrow it with
[`with_services`](Self::with_services) / [`requesting`](Self::requesting)
and [`ignoring`](Self::ignoring).
