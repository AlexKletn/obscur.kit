(module
  (memory (export "memory") 1)
  (global (export "obscur_plugin_buffer_ptr") i32 (i32.const 0))
  (global (export "obscur_plugin_buffer_capacity") i32 (i32.const 65536))

  (func (export "obscur_plugin_initialize_v1") (param i32) (result i32)
    i32.const 0)

  (func (export "obscur_plugin_dispatch_v1") (param $length i32) (result i32)
    local.get $length)

  (func (export "obscur_plugin_deactivate_v1") (param i32) (result i32)
    i32.const 0))
