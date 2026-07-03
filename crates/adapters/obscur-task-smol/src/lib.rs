//! smol-based task port adapter (spec §2.7 — "Опциональные реализации task
//! ports. Ни одна не является core dependency.").
//!
//! smol-backed implementations of the `obscur-task` ports, built directly on
//! the smol ecosystem's building blocks (`async-executor`, `async-io`,
//! `futures-lite`, `async-channel`) rather than the `smol` umbrella crate.
//! Every implementation here is optional: the kit's core never depends on smol,
//! and a host that wires this adapter chooses to bring the smol executor into
//! its reach.
//!
//! Unlike an ambient runtime, an [`async_executor::Executor`] must be driven by
//! a thread calling its `run` method, so [`spawner::SmolSpawner`] owns a worker
//! thread and shares its executor with [`timer::SmolTimer`] so scheduled work
//! actually runs.

#![forbid(unsafe_code)]

pub mod spawner;
pub mod timer;
