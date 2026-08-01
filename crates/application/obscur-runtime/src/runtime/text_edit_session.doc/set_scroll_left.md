Sets the field's horizontal scroll offset (logical px). The runtime clamps
it into `0 ..= max(0, content_width − W)` before calling this, so the paint
and click-inverse read a valid, in-range offset.
