The application's handle onto this runtime. Everything it queued is applied
once per committed frame — see [`AppRunner::render_frame`] for where, and
[`AppControl`] for why it is a queue rather than an accessor.
