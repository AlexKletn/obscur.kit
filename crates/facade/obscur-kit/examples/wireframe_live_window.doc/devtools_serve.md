Serves the DevTools endpoint over `listener` forever, one session at a time:
each answers inspects from the shared, per-frame-refreshed `source` and is
pushed whatever the frame thread put on `events`.

# Scopes

`observer()` = read + subscribe. Deliberately **not** `all()`: control and
injection stay ungranted, so an attached client can watch this window but not
steer it. Subscribe is granted only because this host genuinely services it
(`serve_with_events` below).
