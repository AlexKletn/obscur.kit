§3.5 Rules — "updates are coalesced".

Three external frame requests and two committed transactions between two
frame boundaries must produce **exactly one** frame, and that frame must show
the *later* transaction's state.
