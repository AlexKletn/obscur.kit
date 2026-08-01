Number of linear sub-segments each authored stop pair is split into when
densifying in Oklab. `N` segments insert `N - 1` interior Oklab-interpolated
stops between each authored pair; peniko's per-stop sampling (sRGB-linear
between adjacent stops) then closely approximates the true Oklab ramp. 16 keeps
the mid-range hues visually smooth while staying far under any ramp limit.
