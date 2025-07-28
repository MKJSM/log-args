//! # Span Context Propagation Example
//!
//! This example demonstrates span-based context propagation, which is now
//! **enabled by default** with `#[params]`. This shows how to use selective
//! field propagation and custom fields with the default span behavior.
//!
//! ## Usage
//! ```bash
//! cargo run --example span
//! ```
//!
//! ## Features Demonstrated
//! - **Default span-based context propagation** (no explicit `span` attribute needed)
//! - Selective field propagation with `fields()` attribute
//! - Custom fields in spans
//! - Async function span support
//! - Nested span hierarchies
//! - Context inheritance in child functions

use log_args::params;
use log_args_runtime::{info as info_ctx, warn as warn_ctx};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct RequestContext {
    request_id: String,
    session_id: String,
    ip_address: String,
}

/// Processes user request with span context propagation.
/// All parameters are automatically propagated to child functions.
/// (Span propagation is now enabled by default)
#[params]
fn process_user_request(user: User, context: RequestContext) {
    info!("Starting user request processing");

    // Child functions inherit the span context
    validate_user_permissions(user.role.clone());
    log_user_activity(user.id);

    info!("User request processing completed");
}

/// Child function that receives context from parent span.
fn validate_user_permissions(role: String) {
    info_ctx!("Validating user permissions");

    match role.as_str() {
        "admin" => info_ctx!("Admin permissions granted"),
        "user" => info_ctx!("Standard user permissions granted"),
        _ => warn_ctx!("Unknown role, using default permissions"),
    }
}

/// Another child function that inherits context.
fn log_user_activity(_user_id: u64) {
    info_ctx!("Logging user activity");
    info_ctx!("User activity logged successfully");
}

/// Database operation with selective field propagation.
/// Only specified fields are propagated to child spans.
/// (Span propagation is enabled by default, only fields() needed)
#[params(fields(user.id, user.role, operation))]
fn perform_database_operation(user: User, operation: String, _sensitive_data: String) {
    info!("Starting database operation");

    // Only user.id, user.role, and operation are propagated
    connect_to_database();
    execute_query(operation.clone());

    info!("Database operation completed");
}

/// Child function inheriting selective context.
fn connect_to_database() {
    info_ctx!("Establishing database connection");
    info_ctx!("Database connection established");
}

/// Another child function with inherited selective context.
fn execute_query(_query_type: String) {
    info_ctx!("Executing database query");
    info_ctx!("Query execution completed");
}

/// API request handling with custom fields in span.
/// Custom fields are propagated to all child functions.
/// (Span propagation is enabled by default)
#[params(custom(service = "user-api", version = "2.1.0", environment = "production"))]
fn handle_api_request(endpoint: String, _method: String) {
    info!("Handling API request");

    // Custom fields are propagated to child spans
    validate_request_format();
    process_api_logic(endpoint);
    send_response();

    info!("API request handled successfully");
}

/// Child functions inherit custom fields from parent span.
fn validate_request_format() {
    info_ctx!("Validating request format");
    info_ctx!("Request format is valid");
}

fn process_api_logic(_endpoint: String) {
    info_ctx!("Processing API logic");
    info_ctx!("API logic processing completed");
}

fn send_response() {
    info_ctx!("Sending API response");
    info_ctx!("Response sent successfully");
}

/// Async function with span context propagation.
/// (Span propagation is enabled by default)
#[params(fields(user_id, notification_type))]
async fn send_notification_async(user_id: u64, notification_type: String, _template: String) {
    info!("Starting async notification process");

    // Async child functions also inherit span context
    prepare_notification_data().await;
    deliver_notification(notification_type.clone()).await;

    info!("Async notification process completed");
}

/// Async child function with inherited context.
async fn prepare_notification_data() {
    info_ctx!("Preparing notification data");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info_ctx!("Notification data prepared");
}

/// Another async child function.
async fn deliver_notification(_notification_type: String) {
    info_ctx!("Delivering notification");

    // Simulate async delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info_ctx!("Notification delivered successfully");
}

/// Complex nested span hierarchy example.
/// (Span propagation is enabled by default)
#[params(fields(order_id, customer_id))]
fn process_order(order_id: String, customer_id: u64, _payment_info: String) {
    info!("Starting order processing");

    // Multiple levels of nested function calls
    validate_order();
    process_payment();
    fulfill_order();

    info!("Order processing completed");
}

/// Level 1 child function.
fn validate_order() {
    info_ctx!("Validating order details");

    // This calls another child function
    check_inventory();
    validate_customer_info();

    info_ctx!("Order validation completed");
}

/// Level 2 child functions.
fn check_inventory() {
    info_ctx!("Checking product inventory");
    info_ctx!("Inventory check passed");
}

fn validate_customer_info() {
    info_ctx!("Validating customer information");
    info_ctx!("Customer information is valid");
}

/// Level 1 child function.
fn process_payment() {
    info_ctx!("Processing payment");

    // Another nested call
    charge_payment_method();

    info_ctx!("Payment processed successfully");
}

/// Level 2 child function.
fn charge_payment_method() {
    info_ctx!("Charging payment method");
    info_ctx!("Payment method charged successfully");
}

/// Level 1 child function.
fn fulfill_order() {
    info_ctx!("Fulfilling order");

    // More nested calls
    prepare_shipment();
    send_confirmation();

    info_ctx!("Order fulfillment completed");
}

/// Level 2 child functions.
fn prepare_shipment() {
    info_ctx!("Preparing shipment");
    info_ctx!("Shipment prepared");
}

fn send_confirmation() {
    info_ctx!("Sending order confirmation");
    info_ctx!("Order confirmation sent");
}

/// Order service for demonstrating method span propagation.
struct OrderService;

impl OrderService {
    /// Creates order with span context in method.
    #[params(span, custom(component = "order-service", version = "1.0.0"))]
    fn create_order(&self, _customer_id: u64, _items: Vec<String>) {
        info!("Creating new order");

        // Method calls also propagate span context
        self.validate_items();
        self.calculate_total();
        self.save_to_database();

        info!("Order created successfully");
    }

    fn validate_items(&self) {
        info_ctx!("Validating order items");
        info_ctx!("All items are valid");
    }

    fn calculate_total(&self) {
        info_ctx!("Calculating order total");
        info_ctx!("Order total calculated");
    }

    fn save_to_database(&self) {
        info_ctx!("Saving order to database");
        info_ctx!("Order saved successfully");
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("=== Span Context Propagation Example ===");
    println!("This example demonstrates automatic context propagation to child functions.");
    println!();

    // Example 1: Basic span with all parameters
    let user = User {
        id: 12345,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        role: "admin".to_string(),
    };
    let context = RequestContext {
        request_id: "req-abc123".to_string(),
        session_id: "sess-xyz789".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };
    process_user_request(user.clone(), context);

    println!();

    // Example 2: Span with selective field propagation
    perform_database_operation(
        user,
        "SELECT".to_string(),
        "sensitive_query_data".to_string(),
    );

    println!();

    // Example 3: Span with custom fields
    handle_api_request("/users".to_string(), "GET".to_string());

    println!();

    // Example 4: Async function with span
    send_notification_async(67890, "email".to_string(), "welcome_template".to_string()).await;

    println!();

    // Example 5: Complex nested span hierarchy
    process_order(
        "ORD-789123".to_string(),
        54321,
        "payment_token_xyz".to_string(),
    );

    println!();

    // Example 6: Method with span in impl block
    let order_service = OrderService;
    let items = vec![
        "item-001".to_string(),
        "item-002".to_string(),
        "item-003".to_string(),
    ];
    order_service.create_order(98765, items);

    println!();
    println!("✅ Check the JSON logs above to see context propagation in action!");
    println!("🔗 Notice how child functions inherit parent context automatically.");
}
