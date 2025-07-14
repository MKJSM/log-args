//! log-args-runtime
//!
//! This crate provides runtime support for the `log-args` procedural macro crate.
//! It exposes thread-local storage for propagating structured logging context (such as parent spans)
//! at runtime. This crate is used internally by macro-generated code and is not intended for direct use
//! by end users.
//!
//! # Usage
//!
//! Most users do not need to interact with this crate directly. If you are developing custom integrations,
//! you can use the `__PARENT_LOG_ARGS` thread-local variable to set or retrieve the current parent log context.
//!
//! ```rust
//! use log_args_runtime::__PARENT_LOG_ARGS;
//!
//! __PARENT_LOG_ARGS.with(|parent| {
//!     *parent.borrow_mut() = Some("parent-context".to_string());
//! });
//!
//! __PARENT_LOG_ARGS.with(|parent| {
//!     if let Some(ctx) = &*parent.borrow() {
//!         println!("Current parent context: {}", ctx);
//!     }
//! });
//! ```
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static __PARENT_LOG_ARGS: RefCell<Option<HashMap<String, String>>> = RefCell::new(None);
}

pub struct ParentLogArgsGuard;

pub fn set_parent_log_args(ctx: HashMap<String, String>) -> ParentLogArgsGuard {
    __PARENT_LOG_ARGS.with(|parent| {
        *parent.borrow_mut() = Some(ctx);
    });
    ParentLogArgsGuard
}

impl Drop for ParentLogArgsGuard {
    fn drop(&mut self) {
        __PARENT_LOG_ARGS.with(|parent| {
            *parent.borrow_mut() = None;
        });
    }
}

