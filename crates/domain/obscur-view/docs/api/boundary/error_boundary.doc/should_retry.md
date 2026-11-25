Returns whether a contained failure may still be retried.

A retry is available only while the boundary is enabled and the current
failure's attempt count is below the policy's budget. With no recorded
failure there is nothing to retry.
