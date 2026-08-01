The live interaction state the view is parameterized by. Both drivers own one
behind an `Rc<RefCell<_>>`: the command / text-input handlers mutate it, and the
reactive [`view`] builder reads it back to rebuild every frame.
