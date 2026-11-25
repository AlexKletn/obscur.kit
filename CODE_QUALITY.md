# Code quality agreement

[Русская версия](CODE_QUALITY.ru.md)

- **One entity per file.** Keep a type with its implementations, methods, and internal helpers. Give an independent type, state, resource owner, or handler with its own responsibility a separate file.
- Put tests in a separate `<name>.tests.<ext>` file next to `<name>.<ext>` where that convention applies. For Rust, follow the [crate source layout](specs/obscur-kit-2-crate-architecture/03-source-layout-by-crate.md), which keeps an entity’s unit tests with the entity.
- **Group files that belong to the same domain in a module.** Keep only module declarations and re-exports in `lib.rs` and `mod.rs`.
- **Use names that reveal intent.** Use the same term for the same concept throughout the codebase.
- **Keep functions focused.** Give each function one job.
- **Keep shared rules in one place.** Consolidate logic that must change together.
- **Explain why in comments.** Document constraints and decisions that aren't evident from the code.
- **Make the code self-explanatory.** What the code does should be clear from the code itself.
- **Keep cognitive load low.** Prefer straightforward solutions. Avoid clever tricks that make readers reconstruct the flow of control or data.
- **Do not describe what the code does in comments.**
- **Handle errors where you can act on them.** Preserve the original cause when passing a failure to the caller.
- **Give every resource an owner.** The code should make clear who frees memory, closes a descriptor, or releases a lock.
- **Keep state consistent on failure.** If an operation partially completes, roll back its changes or tell the caller exactly what completed.
- **Define how shared state is accessed.** Specify who can modify shared data, what synchronization is required, and in what order to acquire multiple locks.
- **Make units explicit and check ranges.** Use names or types that distinguish bytes, pages, and cycles. Check for overflow when calculating sizes and offsets.
- **Test failure paths.** Cover partial initialization and failures after acquiring a resource. Check the operation result and that resources are released.
- **Document safety invariants.** Next to each `unsafe` operation, state the conditions that make it safe.
