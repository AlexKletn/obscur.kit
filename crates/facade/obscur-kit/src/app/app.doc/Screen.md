An `.okui` source, compiled to UI IR when [`App::run`] is called (a compile
failure surfaces as an [`AppError`] from `run`) and lowered to a `View` on
every frame through [`OkuiScreen`].
