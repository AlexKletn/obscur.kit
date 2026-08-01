§3.4 Frame pipeline — "The synchronous frame phase must stay short and
deterministic." and §5.15 "Features:" / "- async load/decode/compile;" /
"- loading/ready/failed states;"

NG-ASYNC-01: an image whose load never completes must not block the frame; it
paints its reserved box in `loading`, and no further frame is requested until
the completion arrives.
