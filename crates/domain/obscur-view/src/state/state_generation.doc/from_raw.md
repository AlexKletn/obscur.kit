Wraps a raw generation value.

Intended for reconstructing a previously observed generation (for example
when rehydrating a consumer's last-seen watermark); ordinary code advances
generations with [`StateGeneration::next`].
