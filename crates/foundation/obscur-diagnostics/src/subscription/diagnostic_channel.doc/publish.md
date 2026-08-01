Publishes `diagnostic` to the channel, offering it to every enabled,
matching subscription. Each subscription applies its own buffer policy,
so backpressure is independent per subscriber.
