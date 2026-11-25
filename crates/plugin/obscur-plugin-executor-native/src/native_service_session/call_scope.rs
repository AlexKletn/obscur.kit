use std::{cell::RefCell, rc::Rc};

use obscur_plugin_abi::value::AbiHandle;

use super::Dispatch;

thread_local! {
    static CURRENT: RefCell<Option<(AbiHandle, Rc<Dispatch>)>> = const { RefCell::new(None) };
}

/// Keeps native callbacks tied to the innermost call on the host thread.
pub(super) struct CallScope(Option<(AbiHandle, Rc<Dispatch>)>);

impl CallScope {
    pub(super) fn enter(handle: AbiHandle, dispatch: Rc<Dispatch>) -> Self {
        Self(CURRENT.replace(Some((handle, dispatch))))
    }

    pub(super) fn dispatch_for(handle: AbiHandle) -> Option<Rc<Dispatch>> {
        // Release the thread-local borrow before dispatch so a service can enter a nested call.
        CURRENT.with_borrow(|current| {
            let (owner, dispatch) = current.as_ref()?;
            (*owner == handle).then(|| Rc::clone(dispatch))
        })
    }
}

impl Drop for CallScope {
    fn drop(&mut self) {
        CURRENT.replace(self.0.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwinding_a_nested_call_restores_the_outer_dispatch() {
        let outer = AbiHandle::new(1);
        let inner = AbiHandle::new(2);
        let dispatch: Rc<Dispatch> = Rc::new(|_| Ok(Vec::new()));
        let outer_scope = CallScope::enter(outer, Rc::clone(&dispatch));
        let result = std::panic::catch_unwind(|| {
            let _inner_scope = CallScope::enter(inner, Rc::new(|_| Ok(Vec::new())));
            assert!(CallScope::dispatch_for(outer).is_none());
            assert!(CallScope::dispatch_for(inner).is_some());
            panic!("the nested callback failed");
        });
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"the nested callback failed")
        );
        assert!(CallScope::dispatch_for(inner).is_none());
        assert!(Rc::ptr_eq(
            &CallScope::dispatch_for(outer).unwrap(),
            &dispatch
        ));
        drop(outer_scope);
        assert!(CallScope::dispatch_for(outer).is_none());
    }
}
