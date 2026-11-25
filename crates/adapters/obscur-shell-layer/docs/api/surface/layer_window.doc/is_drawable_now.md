Whether the compositor has since said it is ready to composite this surface.

False the moment the callback is asked for, true when it arrives. A loop that
builds only when this is true builds only frames somebody is waiting for.
