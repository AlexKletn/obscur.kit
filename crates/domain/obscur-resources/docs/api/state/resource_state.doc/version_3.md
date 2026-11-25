Returns the version this state was produced at.

[`ResourceState::Ready`] and [`ResourceState::Failed`] carry one;
[`ResourceState::Pending`] and [`ResourceState::Loading`] return `None`.
