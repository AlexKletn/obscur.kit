Registers `Badge` on `runtime` — the host half of §2.14's static
registration, written the way an application writes it.

# Panics

If the runtime refuses the registration, which for a single component with a
derived id means the id collided with an already-registered one.
