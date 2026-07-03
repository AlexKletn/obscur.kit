# 3. Внутренняя структура исходников

## 3.1 Базовые правила

### Одна основная сущность — один файл

Файл содержит:

- одну основную публичную или доменно значимую сущность;
- её `impl`;
- приватные локальные функции без самостоятельной семантики;
- unit tests этой сущности.

Несколько равноправных публичных сущностей получают отдельные файлы.

### `mod.rs` — только структура

Допустимо:

```rust
mod window;
mod window_event;
mod window_id;
mod window_request;
mod window_state;

pub use window::Window;
pub use window_event::WindowEvent;
pub use window_id::WindowId;
pub use window_request::WindowRequest;
pub use window_state::WindowState;
```

Недопустимо в `mod.rs`:

- structs/enums/traits;
- algorithms;
- constants;
- implementations;
- tests;
- conversion logic.

### `lib.rs`

Содержит только:

- crate-level docs;
- lint attributes;
- module declarations;
- curated public reexports.

### Запрещённые dumping-ground names

- `utils.rs`;
- `helpers.rs`;
- `common.rs`;
- `misc.rs`;
- `types.rs`;
- `models.rs`;
- неуточнённый `manager.rs`.

Используются семантические имена: `frame_scheduler.rs`, `resource_hub.rs`, `focus_scope.rs`, `plugin_activation.rs`.

### Не создавать пустые слои

Типовой набор `model/event/policy/service/port/error` — ориентир, а не обязательный шаблон. Папка появляется только при наличии нескольких логически связанных сущностей.

## 3.2 Generic bounded-context layout

```text
src/
├── lib.rs
├── model/
│   ├── mod.rs
│   └── <entity>.rs
├── event/
│   ├── mod.rs
│   └── <event>.rs
├── policy/
│   ├── mod.rs
│   └── <policy>.rs
├── service/
│   ├── mod.rs
│   └── <application-service>.rs
├── port/
│   ├── mod.rs
│   └── <port>.rs
└── error/
    ├── mod.rs
    └── <error>.rs
```

## 3.3 Foundation

### `obscur-identity`

```text
src/
├── lib.rs
├── id/
│   ├── mod.rs
│   ├── node_id.rs
│   ├── component_id.rs
│   ├── window_id.rs
│   ├── resource_id.rs
│   ├── runtime_id.rs
│   ├── operation_id.rs
│   ├── correlation_id.rs
│   └── namespace_id.rs
├── generation/
│   ├── mod.rs
│   ├── generation.rs
│   └── version_stamp.rs
└── key/
    ├── mod.rs
    ├── stable_key.rs
    └── interned_symbol.rs
```

Plugin-specific IDs остаются в `obscur-plugin-model`, чтобы foundation не знал о plugins.

### `obscur-geometry`

```text
src/
├── lib.rs
├── unit/
│   ├── mod.rs
│   ├── logical_px.rs
│   ├── physical_px.rs
│   ├── scale_factor.rs
│   └── normalized.rs
├── space/
│   ├── mod.rs
│   ├── local_space.rs
│   ├── parent_space.rs
│   ├── window_space.rs
│   ├── screen_space.rs
│   └── physical_space.rs
├── primitive/
│   ├── mod.rs
│   ├── point.rs
│   ├── size.rs
│   ├── rect.rs
│   ├── vector.rs
│   ├── insets.rs
│   ├── edges.rs
│   └── corner_radii.rs
├── transform/
│   ├── mod.rs
│   ├── affine_transform.rs
│   ├── transform_chain.rs
│   └── coordinate_mapper.rs
├── constraint/
│   ├── mod.rs
│   ├── axis_constraint.rs
│   ├── box_constraints.rs
│   └── aspect_ratio.rs
└── snapping/
    ├── mod.rs
    ├── pixel_snap_mode.rs
    └── pixel_snap_policy.rs
```

Coordinate space по возможности кодируется generic marker type, а не runtime enum.

### `obscur-capability`

```text
src/
├── lib.rs
├── capability/
│   ├── mod.rs
│   ├── capability_id.rs
│   ├── capability.rs
│   ├── capability_scope.rs
│   ├── capability_constraint.rs
│   └── capability_snapshot.rs
├── support/
│   ├── mod.rs
│   ├── support_level.rs
│   └── support_reason.rs
├── permission/
│   ├── mod.rs
│   ├── permission_state.rs
│   ├── permission_duration.rs
│   └── permission_decision.rs
└── event/
    ├── mod.rs
    ├── capability_changed.rs
    └── capability_revoked.rs
```

### `obscur-task`

```text
src/
├── lib.rs
├── time/
│   ├── mod.rs
│   ├── instant.rs
│   ├── duration.rs
│   ├── deadline.rs
│   └── clock.rs
├── task/
│   ├── mod.rs
│   ├── task_id.rs
│   ├── task_scope.rs
│   ├── task_priority.rs
│   ├── task_result.rs
│   └── stale_result.rs
├── cancellation/
│   ├── mod.rs
│   ├── cancellation_token.rs
│   ├── cancellation_source.rs
│   └── cancellation_reason.rs
└── port/
    ├── mod.rs
    ├── spawner.rs
    ├── local_spawner.rs
    ├── timer.rs
    └── blocking_executor.rs
```

### `obscur-diagnostics`

```text
src/
├── lib.rs
├── diagnostic/
│   ├── mod.rs
│   ├── diagnostic.rs
│   ├── diagnostic_code.rs
│   ├── diagnostic_severity.rs
│   ├── diagnostic_subsystem.rs
│   ├── diagnostic_context.rs
│   └── recovery_action.rs
├── performance/
│   ├── mod.rs
│   ├── performance_span.rs
│   ├── performance_counter.rs
│   ├── performance_budget.rs
│   └── budget_violation.rs
├── subscription/
│   ├── mod.rs
│   ├── diagnostic_filter.rs
│   ├── diagnostic_subscription.rs
│   └── diagnostic_buffer_policy.rs
└── port/
    ├── mod.rs
    ├── diagnostic_sink.rs
    └── telemetry_sink.rs
```

## 3.4 View и retained tree

### `obscur-view`

```text
src/
├── lib.rs
├── component/
│   ├── mod.rs
│   ├── component.rs
│   ├── component_contract.rs
│   ├── component_factory.rs
│   ├── component_registry.rs
│   ├── component_context.rs
│   └── component_instance.rs
├── lifecycle/
│   ├── mod.rs
│   ├── lifecycle_state.rs
│   ├── mount_context.rs
│   ├── update_context.rs
│   └── unmount_context.rs
├── contract/
│   ├── mod.rs
│   ├── prop_contract.rs
│   ├── event_contract.rs
│   ├── slot_contract.rs
│   ├── child_contract.rs
│   ├── style_part_contract.rs
│   └── capability_contract.rs
├── view/
│   ├── mod.rs
│   ├── view.rs
│   ├── view_node.rs
│   ├── view_fragment.rs
│   ├── view_key.rs
│   ├── view_property.rs
│   └── view_event_binding.rs
├── state/
│   ├── mod.rs
│   ├── local_state.rs
│   ├── read_signal.rs
│   ├── subscription_scope.rs
│   └── state_generation.rs
└── boundary/
    ├── mod.rs
    ├── error_boundary.rs
    ├── error_boundary_policy.rs
    └── component_failure.rs
```

### `obscur-tree`

```text
src/
├── lib.rs
├── node/
│   ├── mod.rs
│   ├── retained_node.rs
│   ├── node_kind.rs
│   ├── node_flags.rs
│   ├── logical_parent.rs
│   ├── visual_parent.rs
│   └── node_generation.rs
├── tree/
│   ├── mod.rs
│   ├── retained_tree.rs
│   ├── tree_snapshot.rs
│   ├── subtree.rs
│   └── secondary_map.rs
├── reconciliation/
│   ├── mod.rs
│   ├── reconciler.rs
│   ├── reconciliation_plan.rs
│   ├── reconciliation_op.rs
│   ├── child_matcher.rs
│   └── identity_match.rs
├── transaction/
│   ├── mod.rs
│   ├── tree_transaction.rs
│   ├── tree_commit.rs
│   └── tree_change_set.rs
├── portal/
│   ├── mod.rs
│   ├── portal.rs
│   └── portal_target.rs
└── layer/
    ├── mod.rs
    ├── layer.rs
    ├── layer_id.rs
    └── stacking_context.rs
```

## 3.5 Interaction и data transfer

### `obscur-interaction`

```text
src/
├── lib.rs
├── event/
│   ├── mod.rs
│   ├── ui_event.rs
│   ├── event_phase.rs
│   ├── event_origin.rs
│   ├── event_propagation.rs
│   ├── dispatch_result.rs
│   └── event_dispatcher.rs
├── pointer/
│   ├── mod.rs
│   ├── pointer_id.rs
│   ├── pointer_device.rs
│   ├── pointer_event.rs
│   ├── pointer_buttons.rs
│   ├── pointer_capture.rs
│   └── pointer_capture_registry.rs
├── hover/
│   ├── mod.rs
│   ├── hover_chain.rs
│   └── hover_transition.rs
├── focus/
│   ├── mod.rs
│   ├── focus_node.rs
│   ├── focus_scope.rs
│   ├── focus_tree.rs
│   ├── focus_request.rs
│   ├── focus_result.rs
│   ├── focus_restore.rs
│   └── tab_order.rs
├── command/
│   ├── mod.rs
│   ├── physical_key.rs
│   ├── logical_key.rs
│   ├── shortcut.rs
│   ├── command.rs
│   ├── command_state.rs
│   └── command_router.rs
├── gesture/
│   ├── mod.rs
│   ├── gesture_arena.rs
│   ├── gesture_contender.rs
│   ├── gesture_resolution.rs
│   └── gesture_recognizer.rs
├── selection/
│   ├── mod.rs
│   ├── selection.rs
│   ├── selection_anchor.rs
│   ├── selection_mode.rs
│   └── selection_policy.rs
├── spatial/
│   ├── mod.rs
│   ├── spatial_index.rs
│   ├── hit_test.rs
│   ├── hit_chain.rs
│   ├── spatial_navigation.rs
│   └── navigation_direction.rs
└── gamepad/
    ├── mod.rs
    ├── gamepad_event.rs
    ├── gamepad_button.rs
    └── haptic_request.rs
```

### `obscur-data-transfer`

```text
src/
├── lib.rs
├── offer/
│   ├── mod.rs
│   ├── data_offer.rs
│   ├── data_representation.rs
│   ├── mime_type.rs
│   └── lazy_data_provider.rs
├── clipboard/
│   ├── mod.rs
│   ├── clipboard_request.rs
│   ├── clipboard_result.rs
│   └── clipboard_provider.rs
├── drag/
│   ├── mod.rs
│   ├── drag_session.rs
│   ├── drag_source.rs
│   ├── drop_target.rs
│   ├── drag_effect.rs
│   ├── drag_preview.rs
│   └── drag_autoscroll.rs
└── transfer/
    ├── mod.rs
    ├── transfer_payload.rs
    ├── transfer_limit.rs
    └── transfer_error.rs
```

## 3.6 Style, CSS и layout

### `obscur-style`

```text
src/
├── lib.rs
├── property/
│   ├── mod.rs
│   ├── property_id.rs
│   ├── property_value.rs
│   ├── inherited_property.rs
│   └── custom_property.rs
├── selector/
│   ├── mod.rs
│   ├── selector_id.rs
│   ├── selector_specificity.rs
│   ├── selector_match.rs
│   └── pseudo_state.rs
├── cascade/
│   ├── mod.rs
│   ├── cascade.rs
│   ├── cascade_origin.rs
│   ├── cascade_priority.rs
│   ├── declaration.rs
│   └── important_flag.rs
├── computed/
│   ├── mod.rs
│   ├── computed_style.rs
│   ├── computed_value.rs
│   ├── style_snapshot.rs
│   └── style_diff.rs
├── scope/
│   ├── mod.rs
│   ├── style_scope.rs
│   ├── module_scope.rs
│   ├── theme_scope.rs
│   └── exported_style_part.rs
└── invalidation/
    ├── mod.rs
    ├── style_dependency.rs
    ├── style_invalidation.rs
    ├── style_invalidation_reason.rs
    └── style_cache.rs
```

### `obscur-css`

```text
src/
├── lib.rs
├── source/
│   ├── mod.rs
│   ├── css_source.rs
│   ├── source_id.rs
│   └── source_span.rs
├── parser/
│   ├── mod.rs
│   ├── stylesheet_parser.rs
│   ├── selector_parser.rs
│   ├── property_parser.rs
│   ├── value_parser.rs
│   ├── media_query_parser.rs
│   └── container_query_parser.rs
├── compiler/
│   ├── mod.rs
│   ├── stylesheet_compiler.rs
│   ├── selector_compiler.rs
│   ├── property_compiler.rs
│   ├── transition_compiler.rs
│   └── keyframe_compiler.rs
└── error/
    ├── mod.rs
    ├── css_parse_error.rs
    └── unsupported_css_feature.rs
```

### `obscur-layout`

```text
src/
├── lib.rs
├── model/
│   ├── mod.rs
│   ├── layout_style.rs
│   ├── layout_box.rs
│   ├── layout_result.rs
│   ├── layout_snapshot.rs
│   └── intrinsic_size.rs
├── context/
│   ├── mod.rs
│   ├── layout_context.rs
│   ├── layout_context_id.rs
│   ├── layout_dependency.rs
│   └── layout_containment.rs
├── algorithm/
│   ├── mod.rs
│   ├── layout_algorithm.rs
│   ├── flow_layout.rs
│   ├── flex_layout.rs
│   ├── grid_layout.rs
│   ├── positioned_layout.rs
│   ├── constraint_layout.rs
│   ├── docking_layout.rs
│   └── custom_layout.rs
├── pass/
│   ├── mod.rs
│   ├── measure_pass.rs
│   ├── arrange_pass.rs
│   └── layout_pass_result.rs
├── invalidation/
│   ├── mod.rs
│   ├── layout_invalidation.rs
│   ├── layout_invalidation_reason.rs
│   ├── propagation_result.rs
│   └── layout_cache.rs
├── observer/
│   ├── mod.rs
│   ├── layout_observer.rs
│   ├── resize_observer.rs
│   ├── visibility_observer.rs
│   ├── intersection_observer.rs
│   └── transform_observer.rs
├── virtualization/
│   ├── mod.rs
│   ├── virtual_layout.rs
│   ├── materialized_range.rs
│   ├── overscan_policy.rs
│   ├── size_estimator.rs
│   └── scroll_anchor.rs
└── port/
    ├── mod.rs
    ├── intrinsic_measure.rs
    └── layout_environment.rs
```

Taffy integration должна оставаться внутренней:

```text
src/algorithm/taffy/
├── mod.rs
├── taffy_style_mapper.rs
├── taffy_tree_view.rs
└── taffy_result_mapper.rs
```

Public API не содержит `taffy::Style` или `taffy::NodeId`.

## 3.7 Text и motion

### `obscur-text`

```text
src/
├── lib.rs
├── font/
│   ├── mod.rs
│   ├── font_family.rs
│   ├── font_attributes.rs
│   ├── font_source.rs
│   ├── font_face.rs
│   ├── font_fallback.rs
│   └── font_provider.rs
├── shaping/
│   ├── mod.rs
│   ├── shaping_request.rs
│   ├── shaped_glyph.rs
│   ├── glyph_run.rs
│   ├── text_direction.rs
│   └── text_shaper.rs
├── layout/
│   ├── mod.rs
│   ├── text_layout.rs
│   ├── text_line.rs
│   ├── line_break.rs
│   ├── text_alignment.rs
│   ├── text_overflow.rs
│   └── text_hit_test.rs
├── span/
│   ├── mod.rs
│   ├── text_span.rs
│   ├── text_style.rs
│   └── text_range.rs
├── selection/
│   ├── mod.rs
│   ├── text_selection.rs
│   ├── caret.rs
│   ├── caret_affinity.rs
│   └── selection_geometry.rs
├── editing/
│   ├── mod.rs
│   ├── text_buffer.rs
│   ├── edit_command.rs
│   ├── edit_transaction.rs
│   ├── deletion_unit.rs
│   ├── navigation_unit.rs
│   └── undo_unit.rs
├── ime/
│   ├── mod.rs
│   ├── composition.rs
│   ├── composition_event.rs
│   ├── preedit.rs
│   ├── input_purpose.rs
│   └── input_hints.rs
└── service/
    ├── mod.rs
    ├── text_service_provider.rs
    ├── text_diagnostic.rs
    └── language_service_request.rs
```

Parley/HarfRust/Skrifa types остаются во внутренних adapters.

### `obscur-motion`

```text
src/
├── lib.rs
├── value/
│   ├── mod.rs
│   ├── motion_value.rs
│   ├── interpolatable.rs
│   └── interpolation_result.rs
├── timeline/
│   ├── mod.rs
│   ├── timeline.rs
│   ├── timeline_id.rs
│   ├── timeline_state.rs
│   └── timeline_position.rs
├── track/
│   ├── mod.rs
│   ├── motion_track.rs
│   ├── keyframe_track.rs
│   ├── transition_track.rs
│   └── spring_track.rs
├── driver/
│   ├── mod.rs
│   ├── motion_driver.rs
│   ├── time_driver.rs
│   ├── progress_driver.rs
│   ├── signal_driver.rs
│   └── scroll_driver.rs
├── transition/
│   ├── mod.rs
│   ├── shared_element_transition.rs
│   ├── layout_transition.rs
│   ├── transition_interruption.rs
│   └── transition_direction.rs
├── phase/
│   ├── mod.rs
│   ├── pre_layout_sample.rs
│   └── post_layout_sample.rs
└── scheduler/
    ├── mod.rs
    ├── motion_scheduler.rs
    └── active_motion_set.rs
```

## 3.8 Resources, Scene/Paint и renderer contract

### `obscur-resources`

```text
src/
├── lib.rs
├── handle/
│   ├── mod.rs
│   ├── resource_handle.rs
│   ├── image_handle.rs
│   ├── font_handle.rs
│   ├── stylesheet_handle.rs
│   └── shader_handle.rs
├── state/
│   ├── mod.rs
│   ├── resource_state.rs
│   ├── resource_version.rs
│   ├── resource_error.rs
│   └── resource_metadata.rs
├── source/
│   ├── mod.rs
│   ├── resource_source.rs
│   ├── embedded_source.rs
│   ├── memory_source.rs
│   ├── document_source.rs
│   └── generated_source.rs
├── hub/
│   ├── mod.rs
│   ├── resource_hub.rs
│   ├── resource_entry.rs
│   ├── resource_namespace.rs
│   └── resource_request.rs
├── cache/
│   ├── mod.rs
│   ├── cache_key.rs
│   ├── cache_budget.rs
│   ├── cache_entry.rs
│   ├── eviction_policy.rs
│   └── cache_statistics.rs
└── port/
    ├── mod.rs
    ├── resource_loader.rs
    ├── resource_decoder.rs
    ├── resource_compiler.rs
    └── cache_store.rs
```

### `obscur-scene`

```text
src/
├── lib.rs
├── color/
│   ├── mod.rs
│   ├── color.rs
│   ├── color_space.rs
│   └── alpha_mode.rs
├── path/
│   ├── mod.rs
│   ├── path.rs
│   ├── path_segment.rs
│   └── fill_rule.rs
├── paint/
│   ├── mod.rs
│   ├── paint.rs
│   ├── brush.rs
│   ├── solid_brush.rs
│   ├── linear_gradient.rs
│   ├── radial_gradient.rs
│   ├── conic_gradient.rs
│   ├── stroke.rs
│   └── blend_mode.rs
├── clip/
│   ├── mod.rs
│   ├── clip.rs
│   ├── clip_shape.rs
│   └── mask.rs
├── layer/
│   ├── mod.rs
│   ├── scene_layer.rs
│   ├── layer_effect.rs
│   └── composition_mode.rs
├── command/
│   ├── mod.rs
│   ├── paint_command.rs
│   ├── display_list.rs
│   └── display_list_builder.rs
├── scene/
│   ├── mod.rs
│   ├── scene.rs
│   ├── scene_snapshot.rs
│   ├── scene_generation.rs
│   └── scene_statistics.rs
└── surface/
    ├── mod.rs
    ├── foreign_surface.rs
    ├── canvas_surface.rs
    └── offscreen_surface.rs
```

### `obscur-render`

```text
src/
├── lib.rs
├── renderer/
│   ├── mod.rs
│   ├── renderer.rs
│   ├── renderer_id.rs
│   ├── renderer_capabilities.rs
│   └── renderer_error.rs
├── target/
│   ├── mod.rs
│   ├── render_target.rs
│   ├── window_target.rs
│   ├── texture_target.rs
│   └── bitmap_target.rs
├── frame/
│   ├── mod.rs
│   ├── render_frame.rs
│   ├── frame_submission.rs
│   ├── present_result.rs
│   └── device_loss.rs
└── extension/
    ├── mod.rs
    ├── render_extension.rs
    ├── custom_pass.rs
    └── trusted_render_context.rs
```

## 3.9 Accessibility, windowing, system, routing

### `obscur-accessibility`

```text
src/
├── lib.rs
├── semantic/
│   ├── mod.rs
│   ├── semantic_node.rs
│   ├── semantic_role.rs
│   ├── semantic_state.rs
│   ├── semantic_value.rs
│   ├── semantic_relation.rs
│   └── semantic_action.rs
├── tree/
│   ├── mod.rs
│   ├── semantic_tree.rs
│   ├── semantic_snapshot.rs
│   └── semantic_update.rs
├── event/
│   ├── mod.rs
│   ├── accessibility_action.rs
│   └── accessibility_focus.rs
└── port/
    ├── mod.rs
    └── accessibility_bridge.rs
```

### `obscur-windowing`

```text
src/
├── lib.rs
├── window/
│   ├── mod.rs
│   ├── window.rs
│   ├── window_state.rs
│   ├── window_properties.rs
│   ├── window_constraints.rs
│   ├── window_relation.rs
│   └── window_event.rs
├── request/
│   ├── mod.rs
│   ├── create_window.rs
│   ├── update_window.rs
│   ├── activate_window.rs
│   ├── close_window.rs
│   └── window_operation_result.rs
├── chrome/
│   ├── mod.rs
│   ├── chrome_mode.rs
│   ├── drag_region.rs
│   ├── resize_region.rs
│   └── system_menu_request.rs
├── display/
│   ├── mod.rs
│   ├── display.rs
│   ├── display_mode.rs
│   ├── display_color_capability.rs
│   └── display_event.rs
├── presentation/
│   ├── mod.rs
│   ├── fullscreen_mode.rs
│   ├── presentation_mode.rs
│   └── sleep_inhibition.rs
└── port/
    ├── mod.rs
    └── window_backend.rs
```

### `obscur-system`

```text
src/
├── lib.rs
├── document/
│   ├── mod.rs
│   ├── document_handle.rs
│   ├── directory_handle.rs
│   ├── document_permission.rs
│   ├── document_metadata.rs
│   └── document_service.rs
├── dialog/
│   ├── mod.rs
│   ├── dialog_request.rs
│   ├── dialog_result.rs
│   └── dialog_provider.rs
├── shell/
│   ├── mod.rs
│   ├── shell_target.rs
│   ├── shell_result.rs
│   └── shell_provider.rs
├── notification/
│   ├── mod.rs
│   ├── notification.rs
│   ├── notification_action.rs
│   └── notification_provider.rs
├── capture/
│   ├── mod.rs
│   ├── capture_request.rs
│   ├── capture_result.rs
│   └── capture_provider.rs
├── privacy/
│   ├── mod.rs
│   ├── privacy_class.rs
│   └── privacy_policy.rs
├── power/
│   ├── mod.rs
│   ├── power_profile.rs
│   └── power_provider.rs
└── network/
    ├── mod.rs
    ├── network_request.rs
    ├── network_response.rs
    ├── credential_handle.rs
    └── network_service.rs
```

Опциональные media modules создаются только при реализации.

### `obscur-routing`

```text
src/
├── lib.rs
├── route/
│   ├── mod.rs
│   ├── route.rs
│   ├── route_pattern.rs
│   ├── route_match.rs
│   ├── route_params.rs
│   └── route_state.rs
├── history/
│   ├── mod.rs
│   ├── navigation_entry.rs
│   ├── navigation_direction.rs
│   └── history_provider.rs
└── router/
    ├── mod.rs
    ├── router.rs
    ├── route_registry.rs
    └── navigation_result.rs
```

## 3.10 Runtime и headless

### `obscur-runtime`

```text
src/
├── lib.rs
├── runtime/
│   ├── mod.rs
│   ├── ui_runtime.rs
│   ├── runtime_config.rs
│   ├── runtime_state.rs
│   ├── runtime_handle.rs
│   └── runtime_event.rs
├── frame/
│   ├── mod.rs
│   ├── frame_id.rs
│   ├── frame_reason.rs
│   ├── frame_request.rs
│   ├── frame_context.rs
│   ├── frame_pipeline.rs
│   └── frame_result.rs
├── scheduler/
│   ├── mod.rs
│   ├── frame_scheduler.rs
│   ├── scheduler_priority.rs
│   ├── scheduler_queue.rs
│   ├── backpressure_policy.rs
│   └── starvation_guard.rs
├── transaction/
│   ├── mod.rs
│   ├── ui_transaction.rs
│   ├── ui_operation.rs
│   ├── transaction_validator.rs
│   ├── transaction_commit.rs
│   └── provisional_transaction.rs
├── invalidation/
│   ├── mod.rs
│   ├── invalidation.rs
│   ├── invalidation_kind.rs
│   ├── invalidation_reason.rs
│   └── invalidation_router.rs
├── orchestration/
│   ├── mod.rs
│   ├── style_coordinator.rs
│   ├── layout_coordinator.rs
│   ├── text_coordinator.rs
│   ├── motion_coordinator.rs
│   ├── scene_coordinator.rs
│   └── render_coordinator.rs
├── lifecycle/
│   ├── mod.rs
│   ├── runtime_lifecycle.rs
│   ├── shutdown_request.rs
│   ├── shutdown_concern.rs
│   └── release_result.rs
├── recovery/
│   ├── mod.rs
│   ├── component_recovery.rs
│   ├── subsystem_recovery.rs
│   └── recovery_result.rs
└── quality/
    ├── mod.rs
    ├── adaptive_quality.rs
    ├── quality_level.rs
    └── quality_policy.rs
```

`orchestration` содержит application services, но не дублирует алгоритмы доменов.

### `obscur-headless`

```text
src/
├── lib.rs
├── host/
│   ├── mod.rs
│   ├── headless_host.rs
│   ├── headless_config.rs
│   └── virtual_window.rs
├── deterministic/
│   ├── mod.rs
│   ├── deterministic_clock.rs
│   ├── deterministic_executor.rs
│   └── completion_queue.rs
├── input/
│   ├── mod.rs
│   ├── synthetic_input.rs
│   └── selector_target.rs
├── snapshot/
│   ├── mod.rs
│   ├── pipeline_snapshot.rs
│   ├── snapshot_stage.rs
│   └── snapshot_normalizer.rs
└── replay/
    ├── mod.rs
    ├── event_recording.rs
    ├── replay_session.rs
    └── replay_result.rs
```

## 3.11 UI IR и `.okui`

### `obscur-ui-ir`

```text
src/
├── lib.rs
├── version/
│   ├── mod.rs
│   ├── ui_ir_version.rs
│   └── ui_ir_version_range.rs
├── document/
│   ├── mod.rs
│   ├── ui_ir_document.rs
│   ├── ui_ir_node.rs
│   ├── ui_ir_property.rs
│   ├── ui_ir_event.rs
│   └── ui_ir_expression.rs
├── source/
│   ├── mod.rs
│   ├── source_map.rs
│   ├── source_location.rs
│   └── source_reference.rs
├── validation/
│   ├── mod.rs
│   ├── ui_ir_validator.rs
│   ├── validation_context.rs
│   ├── validation_result.rs
│   └── validation_error.rs
├── migration/
│   ├── mod.rs
│   ├── ui_ir_migration.rs
│   └── migration_registry.rs
└── codec/
    ├── mod.rs
    ├── ui_ir_encoder.rs
    └── ui_ir_decoder.rs
```

### `obscur-okui-syntax`

```text
src/
├── lib.rs
├── syntax/
│   ├── mod.rs
│   ├── syntax_kind.rs
│   ├── syntax_node.rs
│   ├── syntax_token.rs
│   └── syntax_tree.rs
├── lexer/
│   ├── mod.rs
│   ├── token_kind.rs
│   ├── token.rs
│   └── lexer.rs
├── parser/
│   ├── mod.rs
│   ├── parser.rs
│   ├── parse_event.rs
│   ├── parse_error.rs
│   └── parse_result.rs
└── ast/
    ├── mod.rs
    ├── document.rs
    ├── element.rs
    ├── attribute.rs
    ├── expression.rs
    ├── conditional.rs
    ├── loop_expression.rs
    └── template.rs
```

### `obscur-okui-compiler`

```text
src/
├── lib.rs
├── symbol/
│   ├── mod.rs
│   ├── symbol.rs
│   ├── symbol_table.rs
│   └── scope.rs
├── resolution/
│   ├── mod.rs
│   ├── name_resolver.rs
│   └── resolution_error.rs
├── typing/
│   ├── mod.rs
│   ├── value_type.rs
│   ├── type_checker.rs
│   ├── expression_type.rs
│   └── type_error.rs
├── validation/
│   ├── mod.rs
│   ├── contract_validator.rs
│   ├── expression_validator.rs
│   └── capability_validator.rs
└── lowering/
    ├── mod.rs
    ├── ui_ir_lowering.rs
    ├── expression_lowering.rs
    └── source_map_builder.rs
```

## 3.12 Components

### `obscur-primitives`

```text
src/
├── lib.rs
├── container.rs
├── text.rs
├── image.rs
├── scroll.rs
├── canvas.rs
├── portal.rs
├── layer.rs
└── foreign_surface.rs
```

Если primitive разрастается, он становится модулем:

```text
scroll/
├── mod.rs
├── scroll.rs
├── scroll_state.rs
├── scroll_controller.rs
└── scroll_policy.rs
```

### `obscur-widgets`

```text
src/
├── lib.rs
├── button/
├── input/
├── checkbox/
├── select/
├── menu/
├── dialog/
├── tabs/
├── list/
├── tree/
├── table/
└── docking/
```

Каждый модуль следует one-entity-per-file.

## 3.13 Adapter examples

### `obscur-render-vello`

```text
src/
├── lib.rs
├── renderer/
│   ├── mod.rs
│   ├── vello_renderer.rs
│   ├── vello_device.rs
│   ├── vello_surface.rs
│   └── vello_frame.rs
├── conversion/
│   ├── mod.rs
│   ├── path_converter.rs
│   ├── brush_converter.rs
│   ├── color_converter.rs
│   ├── clip_converter.rs
│   └── scene_converter.rs
├── cache/
│   ├── mod.rs
│   ├── glyph_cache.rs
│   ├── texture_cache.rs
│   └── path_cache.rs
├── extension/
│   ├── mod.rs
│   └── vello_custom_pass.rs
└── error/
    ├── mod.rs
    └── vello_render_error.rs
```

### `obscur-window-winit`

```text
src/
├── lib.rs
├── backend/
│   ├── mod.rs
│   ├── winit_window_backend.rs
│   ├── winit_window.rs
│   └── winit_event_loop.rs
├── conversion/
│   ├── mod.rs
│   ├── window_event_converter.rs
│   ├── keyboard_converter.rs
│   ├── pointer_converter.rs
│   └── display_converter.rs
├── wake/
│   ├── mod.rs
│   └── event_loop_waker.rs
└── error/
    ├── mod.rs
    └── winit_backend_error.rs
```

### `obscur-accessibility-accesskit`

```text
src/
├── lib.rs
├── bridge/
│   ├── mod.rs
│   └── accesskit_bridge.rs
├── conversion/
│   ├── mod.rs
│   ├── role_converter.rs
│   ├── node_converter.rs
│   ├── action_converter.rs
│   └── tree_update_converter.rs
└── error/
    ├── mod.rs
    └── accesskit_bridge_error.rs
```

## 3.14 Plugin system

### `obscur-plugin-model`

```text
src/
├── lib.rs
├── identity/
│   ├── mod.rs
│   ├── plugin_id.rs
│   ├── plugin_version.rs
│   ├── plugin_principal.rs
│   └── contribution_id.rs
├── manifest/
│   ├── mod.rs
│   ├── plugin_manifest.rs
│   ├── plugin_kind.rs
│   ├── entrypoint.rs
│   ├── host_compatibility.rs
│   └── dependency_manifest.rs
├── capability/
│   ├── mod.rs
│   ├── capability_request.rs
│   ├── capability_grant.rs
│   ├── delegated_capability.rs
│   └── user_activation_token.rs
├── contribution/
│   ├── mod.rs
│   ├── contribution.rs
│   ├── widget_contribution.rs
│   ├── screen_contribution.rs
│   ├── route_contribution.rs
│   ├── menu_contribution.rs
│   ├── settings_contribution.rs
│   ├── background_service_contribution.rs
│   └── extension_point.rs
├── lifecycle/
│   ├── mod.rs
│   ├── plugin_lifecycle_state.rs
│   ├── contribution_state.rs
│   ├── shutdown_concern.rs
│   └── failure_kind.rs
├── service/
│   ├── mod.rs
│   ├── service_id.rs
│   ├── service_version.rs
│   ├── service_descriptor.rs
│   └── service_scope.rs
├── message/
│   ├── mod.rs
│   ├── message_envelope.rs
│   ├── query.rs
│   ├── command.rs
│   ├── event.rs
│   └── message_delivery.rs
├── state/
│   ├── mod.rs
│   ├── state_schema.rs
│   ├── state_scope.rs
│   ├── instance_state.rs
│   └── migration_descriptor.rs
└── settings/
    ├── mod.rs
    ├── settings_schema.rs
    ├── setting_field.rs
    ├── setting_value.rs
    └── setting_validation.rs
```

### `obscur-plugin-api`

```text
src/
├── lib.rs
├── host/
│   ├── mod.rs
│   ├── host_api.rs
│   ├── host_service.rs
│   └── host_context.rs
├── plugin/
│   ├── mod.rs
│   ├── plugin_api.rs
│   ├── plugin_context.rs
│   └── plugin_registration.rs
├── handle/
│   ├── mod.rs
│   ├── opaque_handle.rs
│   ├── handle_scope.rs
│   ├── handle_lease.rs
│   └── handle_status.rs
├── executor/
│   ├── mod.rs
│   ├── plugin_executor.rs
│   ├── executor_request.rs
│   ├── executor_event.rs
│   └── executor_status.rs
├── service/
│   ├── mod.rs
│   ├── service_registry.rs
│   ├── service_handle.rs
│   ├── service_request.rs
│   └── service_response.rs
└── status/
    ├── mod.rs
    ├── plugin_status.rs
    └── plugin_api_error.rs
```

### `obscur-plugin-abi`

```text
src/
├── lib.rs
├── function_table/
│   ├── mod.rs
│   ├── host_function_table.rs
│   └── plugin_function_table.rs
├── value/
│   ├── mod.rs
│   ├── abi_string.rs
│   ├── abi_buffer.rs
│   ├── abi_slice.rs
│   ├── abi_handle.rs
│   └── abi_status.rs
├── ownership/
│   ├── mod.rs
│   ├── buffer_owner.rs
│   └── release_callback.rs
├── boundary/
│   ├── mod.rs
│   ├── panic_boundary.rs
│   └── abi_call_guard.rs
└── conversion/
    ├── mod.rs
    ├── host_api_converter.rs
    └── plugin_api_converter.rs
```

### `obscur-plugin-package`

```text
src/
├── lib.rs
├── package/
│   ├── mod.rs
│   ├── plugin_package.rs
│   ├── package_format.rs
│   ├── package_entry.rs
│   └── package_reader.rs
├── path/
│   ├── mod.rs
│   ├── package_path.rs
│   └── path_validation.rs
├── validation/
│   ├── mod.rs
│   ├── package_validator.rs
│   ├── package_limits.rs
│   ├── hash_validator.rs
│   └── entrypoint_validator.rs
├── archive/
│   ├── mod.rs
│   ├── zip_package_reader.rs
│   ├── tar_package_reader.rs
│   └── compression.rs
└── extraction/
    ├── mod.rs
    ├── extraction_plan.rs
    ├── isolated_directory.rs
    └── extracted_entry.rs
```

### `obscur-plugin-host`

```text
src/
├── lib.rs
├── manager/
│   ├── mod.rs
│   ├── plugin_manager.rs
│   ├── plugin_catalog.rs
│   └── plugin_record.rs
├── discovery/
│   ├── mod.rs
│   ├── plugin_discovery.rs
│   └── discovered_plugin.rs
├── dependency/
│   ├── mod.rs
│   ├── dependency_graph.rs
│   ├── dependency_resolver.rs
│   └── dependency_conflict.rs
├── policy/
│   ├── mod.rs
│   ├── host_plugin_policy.rs
│   ├── capability_policy.rs
│   ├── activation_policy.rs
│   └── resource_budget.rs
├── activation/
│   ├── mod.rs
│   ├── plugin_activation.rs
│   ├── activation_plan.rs
│   ├── activation_transaction.rs
│   └── activation_result.rs
├── contribution/
│   ├── mod.rs
│   ├── contribution_registry.rs
│   ├── contribution_instance.rs
│   ├── contribution_context.rs
│   ├── contribution_boundary.rs
│   ├── contribution_chrome.rs
│   └── extension_point_registry.rs
├── capability/
│   ├── mod.rs
│   ├── capability_issuer.rs
│   ├── capability_revoker.rs
│   └── delegation_graph.rs
├── lifecycle/
│   ├── mod.rs
│   ├── plugin_lifecycle.rs
│   ├── plugin_teardown.rs
│   ├── plugin_suspension.rs
│   └── crash_recovery.rs
├── update/
│   ├── mod.rs
│   ├── plugin_update.rs
│   ├── state_migration.rs
│   ├── staged_handover.rs
│   └── rollback_plan.rs
├── storage/
│   ├── mod.rs
│   ├── plugin_storage.rs
│   ├── storage_namespace.rs
│   ├── storage_quota.rs
│   └── orphaned_instance.rs
└── performance/
    ├── mod.rs
    ├── plugin_budget_enforcer.rs
    ├── contribution_budget.rs
    └── plugin_watchdog.rs
```

## 3.15 Tests

Unit tests остаются рядом с сущностью:

```rust
#[cfg(test)]
mod tests {
    // tests only for this entity
}
```

Интеграционные тесты:

```text
tests/
├── contract/
├── integration/
├── regression/
└── fixtures/
```

Compiler UI tests:

```text
tests/ui/
├── pass/
└── fail/
```

Snapshot data не смешивается с production resources.

## 3.16 Автоматические architecture checks

`xtask` должен проверять:

- adapter dependency не появляется в domain;
- Tokio/Winit/wgpu/Vello/Wasmtime отсутствуют в pure core;
- `unsafe` отсутствует вне allowlist;
- нет `utils.rs`, `helpers.rs`, `common.rs`, `types.rs`, `models.rs`;
- `mod.rs` содержит только `mod`, `pub mod`, `use`, `pub use` и атрибуты;
- `lib.rs` не содержит реализацию сущностей;
- нет циклов workspace dependencies;
- внутренние крейты не зависят от `obscur-kit`;
- feature matrix согласована;
- один файл не экспортирует несколько главных сущностей без явного exception marker.
