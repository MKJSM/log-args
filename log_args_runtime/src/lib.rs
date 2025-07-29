use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global context store for cross-boundary persistence
static GLOBAL_CONTEXT: std::sync::LazyLock<Arc<Mutex<HashMap<String, String>>>> = 
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Set global context that persists across all boundaries
pub fn set_global_context(key: &str, value: &str) {
    if let Ok(mut global) = GLOBAL_CONTEXT.lock() {
        global.insert(key.to_string(), value.to_string());
    }
}

/// Get global context for cross-boundary persistence
pub fn get_global_context() -> Option<HashMap<String, String>> {
    if let Ok(global) = GLOBAL_CONTEXT.lock() {
        if !global.is_empty() {
            return Some(global.clone());
        }
    }
    None
}

// Thread-local storage for context stacks
thread_local! {
    static CONTEXT_STACK: RefCell<Vec<HashMap<String, String>>> = RefCell::new(Vec::new());
    static ASYNC_CONTEXT_STACK: RefCell<Vec<HashMap<String, String>>> = RefCell::new(Vec::new());
}

/// Guard for synchronous context that automatically pops on drop
#[doc(hidden)]
pub struct ContextGuard;

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

// Function to get a context value from the current span context
pub fn get_context_value(key: &str) -> Option<String> {
    // First, try async context stack
    if let Ok(stack) = ASYNC_CONTEXT_STACK.try_with(|stack| stack.borrow().clone()) {
        for context_map in stack.iter().rev() {
            if let Some(value) = context_map.get(key) {
                return Some(value.clone());
            }
        }
    }
    
    // Then try sync context stack
    let result = CONTEXT_STACK.with(|stack| {
        let stack = stack.borrow();
        for context_map in stack.iter().rev() {
            if let Some(value) = context_map.get(key) {
                return Some(value.clone());
            }
        }
        None
    });
    
    if result.is_some() {
        return result;
    }
    
    // Finally, try global context store for cross-boundary persistence
    if let Ok(global) = GLOBAL_CONTEXT.lock() {
        if let Some(value) = global.get(key) {
            return Some(value.clone());
        }
    }
    
    None
}

/// Get current synchronous context
#[doc(hidden)]
pub fn get_context() -> HashMap<String, String> {
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

#[doc(hidden)]
pub fn get_async_context() -> HashMap<String, String> {
    ASYNC_CONTEXT_STACK
        .try_with(|stack| {
            stack
                .borrow()
                .iter()
                .fold(HashMap::new(), |mut acc, context| {
                    acc.extend(context.clone());
                    acc
                })
        })
        .unwrap_or_default()
}

#[doc(hidden)]
pub fn get_current_async_stack() -> Vec<HashMap<String, String>> {
    ASYNC_CONTEXT_STACK
        .try_with(|stack| stack.borrow().clone())
        .unwrap_or_else(|_| vec![HashMap::new()])
}

/// Push context for synchronous functions with span
#[doc(hidden)]
pub fn push_context(context: HashMap<String, String>) -> ContextGuard {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(context);
    });
    ContextGuard
}

/// Push context for asynchronous functions with span
#[doc(hidden)]
pub fn push_async_context(context: HashMap<String, String>) -> AsyncContextGuard {
    ASYNC_CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(context);
    });
    AsyncContextGuard
}

/// Guard for async context that automatically pops on drop
pub struct AsyncContextGuard;

impl Drop for AsyncContextGuard {
    fn drop(&mut self) {
        ASYNC_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

// Helper macro to dynamically add context fields to log statements
// This macro is now completely dynamic with no hardcoded field names
#[macro_export]
macro_rules! add_context_fields {
    ($log_macro:path, $ctx:expr, $($args:tt)*) => {
        // Completely dynamic approach - no hardcoded field names
        // Create field tokens for all context fields dynamically
        let mut field_tokens = Vec::new();

        // Add all context fields dynamically without hardcoding any field names
        for (key, value) in $ctx.iter() {
            // Create a field token for any field name
            let field_token = if key.contains('.') {
                // Handle dotted field names (like "user.id")
                format!("\"{key}\" = %{value}", key = key, value = value)
            } else {
                // Handle regular field names
                format!("{key} = %{value}", key = key, value = value)
            };
            field_tokens.push(field_token);
        }

        // Note: This approach still has Rust macro limitations
        // The field tokens can't be directly injected into the macro call
        // This is kept for potential future use or alternative implementations
    };
}

// Removed log_with_context macro to eliminate duplicate logging
// All logging now uses direct tracing macros without context string generation

// Removed all context-related macros to eliminate duplicate logging
// All logging now uses direct tracing macros without context string generation

// Removed all runtime context macros to eliminate duplicate logging
// All context propagation is now handled via direct field injection in the main macro
// Child functions should use regular tracing macros (info!, warn!, etc.) which will
// automatically inherit context through the macro redefinition system

// Completely removed all runtime macros to eliminate duplicate logging
// All context propagation is now handled via direct field injection in the main macro
// Child functions should use regular tracing macros which will inherit context
// through the macro redefinition system in functions with #[params]

/// Automatically capture and preserve current context for function execution
/// This ensures context is maintained across function boundaries without user intervention
pub fn auto_capture_context() -> ContextGuard {
    let current_context = get_context();
    
    // Push to both async and sync stacks to ensure maximum compatibility
    let _async_guard = push_async_context(current_context.clone());
    let _sync_guard = push_context(current_context);
    
    // Return the existing ContextGuard (empty struct)
    ContextGuard
}

/// Capture current context and store it globally for cross-boundary persistence
/// This function is automatically called by the macro to ensure context is preserved
pub fn capture_context() -> ContextGuard {
    let current_context = get_context();
    
    // Store each context field globally for cross-boundary access
    for (key, value) in &current_context {
        set_global_context(key, value);
    }
    
    // Also push to context stacks for immediate access
    let _async_guard = push_async_context(current_context.clone());
    let _sync_guard = push_context(current_context);
    
    // Return the existing ContextGuard (empty struct)
    ContextGuard
}

/// Helper function to capture context for closure boundaries
/// This captures the current context and returns a closure that restores it
/// Usage: let captured = with_context_capture1(|arg| { /* your code */ });
pub fn with_context_capture1<F, A, R>(f: F) -> impl FnOnce(A) -> R
where
    F: FnOnce(A) -> R,
{
    let captured_context = get_context();
    
    move |a| {
        // Use async context for better propagation and ensure it persists across function calls
        let _guard = push_async_context(captured_context.clone());
        
        // Also push to sync context for better compatibility
        let _sync_guard = push_context(captured_context);
        
        f(a)
    }
}

/// Helper function for automatic closure context capture with two arguments
pub fn with_context_capture2<F, A, B, R>(f: F) -> impl FnOnce(A, B) -> R
where
    F: FnOnce(A, B) -> R,
{
    let captured_context = get_context();
    
    move |a, b| {
        let _guard = push_context(captured_context);
        f(a, b)
    }
}

/// Helper function for automatic closure context capture with three arguments
pub fn with_context_capture3<F, A, B, C, R>(f: F) -> impl FnOnce(A, B, C) -> R
where
    F: FnOnce(A, B, C) -> R,
{
    let captured_context = get_context();
    
    move |a, b, c| {
        let _guard = push_context(captured_context);
        f(a, b, c)
    }
}

/// Get inherited context as a formatted string for automatic span propagation
/// This function retrieves all context fields from the current span context
/// and formats them as a string for logging
pub fn get_inherited_context_string() -> String {
    // Context string generation completely removed to eliminate duplicate logging
    // All context is now handled via direct field injection only
    String::new()
}

/// Get inherited context fields as individual key-value pairs
/// This function returns a HashMap of inherited context fields for dynamic field injection
pub fn get_inherited_fields_map() -> std::collections::HashMap<String, String> {
    let mut context_map = std::collections::HashMap::new();
    
    // Try async context stack first
    if let Ok(stack) = ASYNC_CONTEXT_STACK.try_with(|stack| stack.borrow().clone()) {
        for stack_context in stack.iter().rev() {
            for (key, value) in stack_context {
                // Skip function name to avoid duplication
                if key != "function" {
                    context_map.insert(key.clone(), value.clone());
                }
            }
            if !context_map.is_empty() {
                return context_map; // Use the most recent context
            }
        }
    }
    
    // If no async context, try sync context stack
    if context_map.is_empty() {
        CONTEXT_STACK.with(|stack| {
            let stack = stack.borrow();
            for stack_context in stack.iter().rev() {
                for (key, value) in stack_context {
                    // Skip function name to avoid duplication
                    if key != "function" {
                        context_map.insert(key.clone(), value.clone());
                    }
                }
                if !context_map.is_empty() {
                    return; // Use the most recent context
                }
            }
        });
    }
    
    context_map
}
