The phase of propagation an event occupies as it travels the logical tree
(spec §4.7 hit testing → event routing).

Mirrors the classic three-phase model: an event first descends from the root
toward the target ([`Capture`](Self::Capture)), then fires at the target
itself ([`Target`](Self::Target)), then ascends back toward the root
([`Bubble`](Self::Bubble)). The
[`EventDispatcher`](crate::event::EventDispatcher) stamps each step of the
computed path with one of these phases.
