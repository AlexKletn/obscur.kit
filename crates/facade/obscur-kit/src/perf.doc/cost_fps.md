The sustainable FPS implied by the mean frame cost (`1000 / avg_total_ms`)
— the rate the frame *could* be produced at back-to-back. `0.0` when
empty. For the *actual* present rate under a continuous redraw, divide the
frames recorded by the wall-clock interval instead.
