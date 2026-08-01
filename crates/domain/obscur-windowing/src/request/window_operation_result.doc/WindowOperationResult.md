The typed outcome of a §6.2 dynamic window property or window operation.

Spec §6.2 requires that operations return typed results across the capability
categories *native*, *emulated*, *conditional*, and *unsupported*. This enum
models the outcome an application observes after submitting such an
operation, reusing [`SupportLevel`] as the canonical category type: the spec
*conditional* category maps to [`SupportLevel::Limited`], and *unsupported*
maps to the distinct [`WindowOperationResult::Unsupported`] variant
(conceptually [`SupportLevel::Unavailable`]).
