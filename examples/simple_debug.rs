use log_args::params;
use log_args_runtime::{get_context_value, set_global_context};
use tracing::info;

/// Test function that sets global context
#[params(span, custom(id = "test_123"))]
pub fn parent_function() {
    // Manually test global context
    set_global_context("manual_test", "manual_value");

    info!("Parent function: Setting context");
    println!("Parent function: Setting context");
    child_function();
}

/// Test function that inherits context
#[params]
pub fn child_function() {
    info!("Child function: Checking context");
    println!("Child function: Checking context");

    // Debug: Check if we can retrieve the manual test value
    let manual_value = get_context_value("manual_test");
    println!("Manual test value: {manual_value:?}");

    // Debug: Check if we can retrieve the id
    let id_value = get_context_value("id");
    println!("ID value: {id_value:?}");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).json().init();

    println!("=== Testing global context mechanism ===");
    parent_function();
    println!("=== Test completed ===");
}
