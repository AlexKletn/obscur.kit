Commit the layer surface, and count it.

Every commit this process makes goes through here, so that a presentation
feedback can be told whether its frame was overwritten by one of them.
