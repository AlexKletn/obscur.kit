§4.9: "Virtualized layout supports:" / "- scroll anchoring;" /
"- estimated geometry;"

100 items of 50px (content extent 5000px), viewport 500px, scrolled to 2500
so item 50 is flush with the viewport top. Growing item 10 (above the
viewport, at content y 500..550) from 50px to 150px must keep item 50 flush:
the offset becomes 2600, the extent 5100, and the visible items stay 50..59.
