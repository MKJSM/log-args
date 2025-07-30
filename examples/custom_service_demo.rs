use log_args::params;
use serde::Serialize;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Serialize)]
struct Order {
    id: u64,
    customer_id: u64,
    amount: f64,
}

#[derive(Debug, Clone, Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .init();

    println!("=== Custom Service Demo ===");
    println!("This example demonstrates custom field logging for service identification and versioning.");

    let order = Order {
        id: 12345,
        customer_id: 67890,
        amount: 299.99,
    };

    let user = User {
        id: 67890,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    // Simulate order processing workflow
    process_order(order.clone());
    validate_payment(order.clone(), user.clone());
    send_confirmation(user);

    println!("\n✅ Check the JSON logs above to see custom service fields!");
}

/// Order service with custom component and version fields
#[params(span, custom(component = "order-service", version = "1.0.0", environment = "production"))]
fn process_order(order: Order) {
    info!("Processing order");
    
    // Simulate some processing
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Call child function - it will inherit the custom fields
    validate_order_details(order);
}

/// Child function inherits parent's custom fields automatically
#[params(fields(order.id, order.amount))]
fn validate_order_details(order: Order) {
    info!("Validating order details");
    
    if order.amount > 1000.0 {
        warn!("High value order detected");
    }
}

/// Payment service with different custom fields
#[params(span, custom(component = "payment-service", version = "2.1.0", team = "payments"))]
fn validate_payment(order: Order, user: User) {
    info!("Validating payment");
    
    // Call child function - inherits payment service context
    check_user_credit_limit(user, order.amount);
}

/// Child function inherits payment service context
#[params(fields(user.id, credit_limit))]
fn check_user_credit_limit(user: User, credit_limit: f64) {
    info!("Checking user credit limit");
    
    if credit_limit > 500.0 {
        warn!("Credit limit check required");
    }
}

/// Notification service with custom fields
#[params(custom(component = "notification-service", version = "1.5.0", channel = "email"))]
fn send_confirmation(user: User) {
    info!("Sending order confirmation");
    
    // Simulate notification sending
    if user.email.contains("@example.com") {
        warn!("Test email domain detected");
    }
}
