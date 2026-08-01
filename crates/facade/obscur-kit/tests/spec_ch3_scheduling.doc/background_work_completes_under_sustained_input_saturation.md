§3.6 The scheduler supports — "starvation prevention for background work".

Under 300 consecutive frames of input saturation, queued background work must
still complete. The spec gives no bound, so the discriminator is
finite-vs-never.
