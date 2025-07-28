//! # All Parameters Logging Example
//!
//! This example demonstrates the `all` attribute for explicitly logging
//! all function parameters. This is useful for debugging or when you want
//! to ensure all parameters are logged regardless of other attributes.
//!
//! ## Usage
//! ```bash
//! cargo run --example all
//! ```
//!
//! ## Features Demonstrated
//! - Explicit all parameter logging with `#[params(all)]`
//! - **Default span-based context propagation** (enabled automatically)
//! - **Function name logging** (when Cargo feature is enabled)
//! - Combination of `all` with other attributes
//! - Async function support with all parameters

use log_args::params;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Config {
    timeout: u32,
    retries: u8,
    debug_mode: bool,
}

/// Debug function that logs all parameters explicitly.
/// The `all` attribute ensures all parameters are logged.
#[params(all)]
fn debug_user_operation(user: User, config: Config, operation_id: String) {
    info!("Starting debug operation");

    // All parameters (user, config, operation_id) are logged automatically
    // This is useful for debugging complex functions

    info!("Debug operation completed");
}

/// Async function with all parameters logging.
#[params(all)]
async fn async_debug_operation(user_id: u64, data: String, metadata: Vec<String>) {
    info!("Starting async debug operation");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info!("Async debug operation completed");
}

/// Function combining `all` with custom fields.
/// This logs all parameters AND adds custom metadata.
#[params(all, custom(service = "debug-service", level = "verbose"))]
fn comprehensive_debug(user: User, request_data: String, flags: Vec<bool>) {
    info!("Starting comprehensive debug");

    // Both all parameters and custom fields are logged
    // This provides maximum observability

    info!("Comprehensive debug completed");
}

/// Function with selective fields despite having many parameters.
/// This shows how `fields()` overrides the default behavior.
#[params(fields(user.id, operation_type))]
fn selective_logging(
    user: User,
    operation_type: String,
    _sensitive_token: String,
    _internal_data: Vec<u8>,
) {
    info!("Starting selective operation");

    // Only user.id and operation_type are logged
    // sensitive_token and internal_data are excluded for security

    info!("Selective operation completed");
}

#[tokio::main]
async fn main() {
    // Initialize structured JSON logging
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .init();

    println!("=== All Parameters Logging Example ===");
    println!("This example demonstrates explicit all parameter logging with the 'all' attribute.");
    println!();

    // Example 1: Basic all parameters logging
    let user = User {
        id: 12345,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    let config = Config {
        timeout: 30,
        retries: 3,
        debug_mode: true,
    };

    debug_user_operation(user.clone(), config, "debug-op-001".to_string());

    println!();

    // Example 2: Async function with all parameters
    async_debug_operation(
        67890,
        "async test data".to_string(),
        vec!["meta1".to_string(), "meta2".to_string()],
    )
    .await;

    println!();

    // Example 3: All parameters with custom fields
    comprehensive_debug(
        user.clone(),
        "comprehensive test".to_string(),
        vec![true, false, true],
    );

    println!();

    // Example 4: Selective logging for comparison
    selective_logging(
        user,
        "selective-op".to_string(),
        "secret-token-12345".to_string(),
        vec![0x01, 0x02, 0x03],
    );

    println!();
    println!("✅ Check the JSON logs above to see the difference between:");
    println!("   - All parameters logging with #[params(all)]");
    println!("   - Default behavior (all params with span propagation)");
    println!("   - Selective logging with #[params(fields(...))]");
    println!("   - Custom fields combined with all parameters");
}
