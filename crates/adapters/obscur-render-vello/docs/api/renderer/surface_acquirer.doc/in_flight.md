Whether an acquire has been asked for and has not yet finished.

Set by the loop when the request is sent, cleared **by the acquire thread** when
`get_current_texture` returns. Only the thread running the acquire can say the
acquire is over, and every version of this that let the other side clear it
produced a second `get_current_texture` while the first image was still out.
