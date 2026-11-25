Fixtures: a runtime already in the state a test wants to start from.

Each fixture renders one frame, so the retained tree, the computed styles and the
layout boxes all exist by the time the test gets it — the precondition for every
node id and every pointer coordinate. The fields are public and the tests
destructure them, which keeps a test's first line a statement of what it needs
rather than a tuple whose fourth element is a `bool` nobody can name.
