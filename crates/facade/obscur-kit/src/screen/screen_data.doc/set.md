Binds `name` to `value`, replacing any previous value.

Takes `&self`: every clone of the handle names one table, so a handler that
captured a clone writes what the next frame reads. Writing does **not**
schedule a frame — the runtime schedules one for the input that ran the
handler, and a write from outside a handler pairs with
[`AppControl::request_frame`](crate::control::AppControl::request_frame).
