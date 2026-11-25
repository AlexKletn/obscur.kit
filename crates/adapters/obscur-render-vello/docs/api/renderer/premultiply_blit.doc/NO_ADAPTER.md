What a run says when it was asked for these tests on a machine that cannot run
them.

The message exists so the failure is self-explaining rather than a bare
`called Option::unwrap on a None value`: it names what is missing, why the test
cannot answer without it, and the two correct ways out (run where an adapter
exists, or leave the tests ignored).
