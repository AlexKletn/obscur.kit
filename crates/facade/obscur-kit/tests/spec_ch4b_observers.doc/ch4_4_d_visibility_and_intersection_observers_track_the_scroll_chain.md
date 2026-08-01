§4.4: "Supported observers:" / "- `VisibilityObserver`;" /
"- `IntersectionObserver`;"

A 200px-tall scroll container holds 20 items of 100px. Both observer types
are registered on item 15. At scroll offset 0 item 15 is neither
intersecting nor visible; after scrolling to 1400 (item 14 flush with the
container top, item 15 at viewport y 100..200) both report
intersecting/visible, and the intersection covers item 15's full 100px.
