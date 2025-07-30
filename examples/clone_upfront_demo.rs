use log_args::params;
use serde::Serialize;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Clone)]
struct NonSerializableType {
    data: String,
}

fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .init();

    println!("=== Clone Upfront Demo ===");
    println!("This example demonstrates the clone_upfront attribute to avoid borrow checker issues.");

    let user = User {
        id: 123,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    let non_serializable = NonSerializableType {
        data: "some data".to_string(),
    };

    let arc_data = Arc::new("shared data".to_string());

    // Test without clone_upfront (normal behavior)
    test_normal_logging(user.clone(), non_serializable.clone(), arc_data.clone());

    // Test with clone_upfront (helps with borrow checker)
    test_clone_upfront_logging(user, non_serializable, arc_data);

    println!("\n✅ Check the JSON logs above to see both normal and clone_upfront behavior!");
}

/// Function without clone_upfront - normal behavior
#[params(all)]
fn test_normal_logging(user: User, non_serializable: NonSerializableType, arc_data: Arc<String>) {
    info!("Normal logging - parameters referenced");
    
    // This would cause borrow checker issues if we tried to use the parameters after logging
    // But since we're not using them, it's fine
}

/// Function with clone_upfront - clones parameters upfront to avoid borrow checker issues
#[params(clone_upfront, all)]
fn test_clone_upfront_logging(user: User, non_serializable: NonSerializableType, arc_data: Arc<String>) {
    info!("Clone upfront logging - parameters cloned before use");
    
    // With clone_upfront, we can safely use the parameters after logging
    // because the macro cloned them before serialization
    let user_name = user.name;
    let data_content = non_serializable.data;
    let shared_content = (*arc_data).clone();
    
    warn!("Using parameters after logging: user={}, data={}, shared={}", 
          user_name, data_content, shared_content);
}
