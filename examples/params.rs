//! # Basic Parameter Logging Example
//!
//! This example demonstrates the basic usage of the `#[params]` macro for span propagation
//! and function name logging with structured tracing support.
//!
//! **By default, `#[params]` only enables span-based context propagation and function name logging.**
//! Parameters are NOT logged automatically - you must use `fields()` or `all` to log parameters.
//!
//! ## Key Features Demonstrated:
//! - Span-based context propagation to child functions (enabled by default)
//! - Function name logging (when enabled via Cargo features)
//! - Selective parameter logging with `fields()` attribute
//! - All parameter logging with `all` attribute
//! - JSON structured logging output
//!
//! ## Usage:
//! ```bash
//! cargo run --example params
//! ```
//!
//! ## Features Demonstrated
//! - **Span-based context propagation (enabled by default)**
//! - **Function name logging (when Cargo feature is enabled)**
//! - Explicit parameter logging with `fields()` attribute
//! - All parameter logging with `all` attribute
//! - Support for sync and async functions
//! - Support for methods in impl blocks
//! - JSON structured logging output
//!
//! ## Function Name Logging
//! To enable function name logging, build with:
//! ```bash
//! cargo run --example params --features function-names-pascal
//! ```

use log_args::params;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    id: u64,
    name: String,
    email: String,
}

/// Authenticates a user with span propagation only (default behavior).
/// Parameters are NOT logged automatically - only span propagation is enabled.
#[params]
fn authenticate_user(_user: User, password: String, _remember_me: bool) {
    info!("Starting user authentication");

    // Your authentication logic here
    if password == "correct_password" {
        info!("Authentication successful");
    } else {
        info!("Authentication failed");
    }
}

/// Async function with selective parameter logging using fields().
#[params(fields(user_id))]
async fn fetch_user_profile(user_id: u64, _include_settings: bool) {
    info!("Fetching user profile");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info!("Profile fetch completed");
}

/// Function with all parameters logged explicitly using the 'all' attribute.
#[params(all)]
fn process_payment(user_id: u64, amount: f64, currency: String) {
    info!("Processing payment");
    
    // Payment processing logic here
    info!("Payment processed successfully");
}

/// Service struct for demonstrating method parameter logging.
struct UserService;

impl UserService {
    /// Creates a new user with span propagation only (default behavior).
    #[params]
    fn create_user(&self, _name: String, _email: String, _age: u32) {
        info!("Creating new user");
        // User creation logic here
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("=== Basic Parameter Logging Example ===");
    println!("This example demonstrates span propagation and selective parameter logging.");
    println!();

    // Example 1: Basic function with span propagation only (default behavior)
    let user = User {
        id: 123,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    authenticate_user(user, "correct_password".to_string(), true);

    println!();

    // Example 2: Async function with selective parameter logging (user_id field)
    fetch_user_profile(456, false).await;

    println!();

    // Example 3: Function with all parameters logged explicitly
    process_payment(789, 99.99, "USD".to_string());

    println!();

    // Example 4: Method call with span propagation only
    let service = UserService;
    service.create_user("Bob Smith".to_string(), "bob@example.com".to_string(), 30);

    println!();
    println!("✅ Check the JSON logs above to see span propagation, selective parameter logging, and all-parameter logging!");
}
