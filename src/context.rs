use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;

// Thread-local storage for the context stack.
thread_local! {
    static CONTEXT_STACK: RefCell<Vec<HashMap<String, Value>>> = RefCell::new(vec![HashMap::new()]);
}

/// A guard that manages the context for a function's lifetime.
#[doc(hidden)]
pub struct ContextGuard;

impl ContextGuard {
    pub fn new(context: HashMap<String, Value>) -> Self {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(context);
        });
        ContextGuard
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// Returns a merged view of the current context stack.
#[doc(hidden)]
pub fn get_context() -> HashMap<String, Value> {
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow()
            .iter()
            .fold(HashMap::new(), |mut acc, context| {
                acc.extend(context.clone());
                acc
            })
    })
}
