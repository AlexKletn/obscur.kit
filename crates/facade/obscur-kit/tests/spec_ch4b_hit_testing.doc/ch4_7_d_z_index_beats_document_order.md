§4.7: "Hit testing accounts for:" / "- layers and z-order;"

Two absolutely-positioned siblings share the rect (0,0,100,100): A declared
first with `z-index: 2`, B declared second with `z-index: 1`. A hit at
(50, 50) must find A on top, not B (which merely comes last in document
order).
