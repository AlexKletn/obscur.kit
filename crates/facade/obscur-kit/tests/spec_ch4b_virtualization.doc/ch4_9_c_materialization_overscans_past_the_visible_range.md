§4.9: "Virtualized layout supports:" / "- materialization;" / "- overscan;"

1000 items of 50px, viewport 500px, offset 0. Items 0..9 intersect the
viewport. The materialized range must cover them, extend **strictly past**
index 9 in the scroll direction (or blank rows pop on every scroll), and
stay far below 1000 (no virtualization at all).
