[`UiRuntime`]'s readings of what work is outstanding: frames the scheduler has
been asked for, input queued but not yet dispatched, and background units waiting
behind them — including §3.6's starvation signal.

The three queues are read together because a host loop asks them together: it
pulls a frame while there is one, and it needs to know that a background unit is
being starved by the frames it is pulling. `take_input_dispatches` is the other
end of the same queue — what dispatch left behind for the caller that queued it.
