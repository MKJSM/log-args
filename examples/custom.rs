//! # Custom Fields Example
//!
//! This example demonstrates the `custom` attribute for adding static fields to logs.
//! The `custom` attribute allows you to add static key-value pairs without needing
//! to pass them as function parameters.
//!
//! ## Usage
//! ```bash
//! cargo run --example custom
//! ```
//!
//! ## Features Demonstrated
//! - Adding static custom fields to logs
//! - Service metadata and operational context
//! - Support for sync and async functions
//! - Support for methods in impl blocks
//! - Multiple data types in custom fields

use log_args::params;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
#[allow(dead_code)]
struct ApiRequest {
    endpoint: String,
    method: String,
    user_id: Option<u64>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
}

/// Handles API requests with service metadata in logs.
/// Custom fields provide consistent context across all log entries.
#[params(custom(service = "user-api", version = "1.2.3", environment = "production"))]
fn handle_api_request(_request: ApiRequest) {
    info!("Processing API request");
    info!("Request completed successfully");
}

/// Connects to database with operational context.
#[params(custom(
    component = "database",
    operation_type = "connection",
    retry_enabled = true,
    timeout_ms = 5000
))]
fn connect_to_database(_config: DatabaseConfig) {
    info!("Attempting database connection");
    info!("Database connection established");
}

/// Sends notifications with module and priority context.
#[params(custom(module = "notification", priority = "high", channel = "email"))]
async fn send_notification(_user_id: u64, _message: String) {
    info!("Preparing notification");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info!("Notification sent successfully");
}

/// Payment processor service.
struct PaymentProcessor;

impl PaymentProcessor {
    /// Processes payments with payment gateway context.
    #[params(custom(
        service = "payment-gateway",
        provider = "stripe",
        currency = "USD",
        secure = true
    ))]
    fn process_payment(&self, _amount: f64, _card_token: String) {
        info!("Processing payment");
        info!("Payment processed successfully");
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("=== Custom Fields Example ===");
    println!("This example demonstrates adding static custom fields to logs.");
    println!();

    // Example 1: API request with service metadata
    let api_request = ApiRequest {
        endpoint: "/users/profile".to_string(),
        method: "GET".to_string(),
        user_id: Some(12345),
    };
    handle_api_request(api_request);

    println!();

    // Example 2: Database connection with operational context
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "user_db".to_string(),
    };
    connect_to_database(db_config);

    println!();

    // Example 3: Async notification with custom fields
    send_notification(67890, "Welcome to our service!".to_string()).await;

    println!();

    // Example 4: Payment processing with custom context
    let processor = PaymentProcessor;
    processor.process_payment(1250.00, "tok_1234567890".to_string());

    println!();
    println!("✅ Check the JSON logs above to see custom fields added to each log entry!");
}
