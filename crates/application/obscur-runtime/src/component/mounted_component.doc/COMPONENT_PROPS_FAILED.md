The code every component-render failure is reported under (spec §1.6: codes
are stable and not reused for unrelated conditions).

One code covers every failure a render can report today, because there is
exactly one: a node's authored properties not decoding into the component's
typed props. A render itself cannot fail — see [`ErasedComponent`].
