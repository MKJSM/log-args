use std::cell::RefCell;

thread_local! {
    pub static __PARENT_LOG_ARGS: RefCell<Option<String>> = RefCell::new(None);
}
