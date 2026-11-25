Replays commands onto the scene under the base transform.

When a filter sink is provided, an extractable blur or color-matrix layer
becomes an independent scene and the sink receives it at its paint position.
The layer clip applies to the input of the filter pass.
Inside an existing clip or layer, or when no sink is provided, drawing commands
stay inside their original group and the filter is omitted.
