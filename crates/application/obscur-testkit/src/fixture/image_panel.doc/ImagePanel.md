A 40×30 element whose background is `url("x")`, and the same box carrying an
element `src` instead.

Two ways in, one question out: a `background-image: url(…)` and an `<img>`
element's own `src` travel different resolution paths and must both end as
exactly one `DrawImage` over the element's box — or, when the load fails, as none
at all. [`src_view`](Self::src_view) deliberately wears the `counter` class so
[`CounterPanel::sheet`](crate::fixture::CounterPanel::sheet) sizes and fills it
*without* a background image, which keeps the only painted image the element's
own.
