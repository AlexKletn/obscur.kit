Ask the compositor to say when this surface is about to be composited again.

The request rides the next commit, which is the frame. What comes back — or does
not — is the only thing in the protocol that distinguishes a surface being drawn
from a surface being ignored.
