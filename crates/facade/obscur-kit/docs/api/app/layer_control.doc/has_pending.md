Whether a reconfiguration is queued and has not been applied yet.

Asked by the loop when it decides how long to sleep. The loop applies pending
settings at the top of a turn, so a request queued during a turn has to be able
to reach the next one — and a loop that slept indefinitely with one queued would
leave a surface that asked to move sitting where it was.
