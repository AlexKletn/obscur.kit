The distance in BYTES from one row's first pixel to the next row's.

Not `width * bytes_per_pixel`. Nearly every real producer pads rows to an
alignment, and reading a padded buffer as a tight one shears the picture.
