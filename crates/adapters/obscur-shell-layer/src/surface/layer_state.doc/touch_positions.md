The last position of each live touch contact, because `wl_touch.up`
carries an id and a time but **no coordinates** — and an `Up` at the
origin would read as a drag to the top-left corner.
