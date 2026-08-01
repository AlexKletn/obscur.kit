The content locale's BCP-47 language hint, passed to every shaping
request (spec §7.2). Configuration, like the ports beside it — not a
per-call input — so it is *not* in [`ParagraphKey`];
[`set_language`](Self::set_language) flushes the cache instead, exactly as
a port swap does.
