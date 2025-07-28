use log_args::params;
use log_args_runtime::{set_global_context, get_context_value};
use tracing::info;

/// Test function that sets global context
#[params(span, custom(company_id = "test_company_123".to_string()))]
pub fn parent_function() {
    // Manually test global context
    set_global_context("manual_test", "manual_value");
    
    println!("Parent function: Setting context");
    child_function();
}

/// Test function that should inherit context
#[params]
pub fn child_function() {
    println!("Child function: Checking context");
    
    // Debug: Check if we can retrieve the manual test value
    let manual_value = get_context_value("manual_test");
    println!("Manual test value: {:?}", manual_value);
    
    // Debug: Check if we can retrieve the company_id
    let company_id = get_context_value("company_id");
    println!("Company ID value: {:?}", company_id);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .json()
        .init();

    println!("=== Testing global context mechanism ===");
    parent_function();
    println!("=== Test completed ===");
}
