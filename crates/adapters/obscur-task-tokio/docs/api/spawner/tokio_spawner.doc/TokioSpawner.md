A [`Spawner`] that schedules `Send` futures onto a Tokio runtime through a
[`Handle`] (spec §2.7 optional task ports).

The handle is cheap to clone and refers to a runtime the host owns; this
adapter merely forwards work to it and never drives the runtime itself.
