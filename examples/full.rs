//! # Comprehensive log_args Features Example
//!
//! This example demonstrates all features of the log_args macro library in a
//! single comprehensive showcase, including real-world usage patterns and
//! security best practices.
//!
//! ## Usage
//! ```bash
//! cargo run --example full
//! ```
//!
//! ## Features Demonstrated
//! - Basic parameter logging with `#[params]`
//! - Selective field logging with `#[params(fields(...))]`
//! - Custom static fields with `#[params(custom(...))]`
//! - Span context propagation with `#[params(span)]`
//! - Combined usage of multiple attributes
//! - Async function support
//! - Error handling and Result types
//! - Method support in impl blocks
//! - Security-conscious logging patterns

use log_args::params;
use log_args_runtime::{info as ctx_info, warn as ctx_warn};
use std::collections::HashMap;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String, // Sensitive - should be excluded from logs
    profile: UserProfile,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserProfile {
    first_name: String,
    last_name: String,
    preferences: UserPreferences,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserPreferences {
    theme: String,
    language: String,
    notifications_enabled: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ApiKey {
    key_id: String,
    secret: String, // Sensitive
    permissions: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct UserOperation {
    operation_type: String,
    priority: u8,
    metadata: HashMap<String, String>,
}

/// Example 1: Basic parameter logging - logs all parameters automatically.
#[params]
fn basic_user_validation(user: User, validation_rules: Vec<String>) {
    info!("Starting basic user validation");

    if user.username.is_empty() {
        error!("Username cannot be empty");
        return;
    }

    info!("Basic user validation completed successfully");
}

/// Example 2: Selective field logging - only logs safe, relevant fields.
/// Excludes sensitive data like passwords and API keys.
#[params(fields(
    user.id,
    user.username,
    user.profile.first_name,
    user.profile.preferences.theme,
    operation_type,
    include_sensitive
))]
fn update_user_profile(
    user: User,
    _new_email: String, // Not logged - could be sensitive during update
    _password: String,  // Not logged - sensitive
    operation_type: String,
    include_sensitive: bool,
    _api_key: ApiKey, // Not logged - contains sensitive data
) {
    info!("Starting user profile update");

    if include_sensitive {
        warn!("Including sensitive data in profile update");
    }

    info!("User profile update completed");
}

/// Example 3: Custom fields - adds static metadata to logs for service identification.
#[params(custom(
    service = "user-management",
    version = "2.1.0",
    environment = "production",
    component = "authentication",
    security_level = "high"
))]
fn authenticate_user_with_metadata(
    username: String,
    password_hash: String, // This parameter will be logged, but it's already hashed
    _remember_me: bool,
) {
    info!("Starting user authentication with metadata");

    if username.is_empty() {
        error!("Authentication failed: empty username");
        return;
    }

    if password_hash.len() < 32 {
        error!("Authentication failed: invalid password hash");
        return;
    }

    info!("User authentication completed successfully");
}

/// Example 4: Span context propagation - creates span and propagates to child functions.
/// Only specified fields are propagated to maintain security.
#[params(span, fields(
    user.id,
    user.username,
    operation.operation_type,
    operation.priority
))]
fn process_user_operation(
    user: User,
    operation: UserOperation,
    _sensitive_token: String, // Not logged or propagated
) {
    info!("Starting user operation processing");

    // These child functions will inherit the span context
    validate_operation_permissions();
    execute_database_operation();
    log_operation_audit();

    info!("User operation processing completed");
}

// Child functions that inherit span context
fn validate_operation_permissions() {
    ctx_info!("Validating operation permissions");
    ctx_info!("Operation permissions validated");
}

fn execute_database_operation() {
    ctx_info!("Executing database operation");
    ctx_warn!("Database operation may take longer than usual");
    ctx_info!("Database operation completed");
}

fn log_operation_audit() {
    ctx_info!("Logging operation for audit");
    ctx_info!("Operation audit logged");
}

/// Example 5: Combined attributes - span + custom + fields together.
#[params(
    span,
    custom(
        service = "payment-processor",
        version = "3.0.1",
        environment = "production",
        compliance = "PCI-DSS"
    ),
    fields(
        transaction_id,
        amount,
        currency,
        user.id,
        fraud_check_enabled
    )
)]
fn process_payment_transaction(
    transaction_id: String,
    amount: f64,
    currency: String,
    user: User,
    _payment_method: String, // Not logged - contains sensitive card data
    fraud_check_enabled: bool,
    _internal_key: String, // Not logged - sensitive
) {
    info!("Starting payment transaction processing");

    // Child functions inherit span context with custom fields
    validate_payment_method();
    charge_payment();
    send_receipt();

    info!("Payment transaction processing completed");
}

// Child functions for payment processing
fn validate_payment_method() {
    ctx_info!("Validating payment method");
    ctx_info!("Payment method validation completed");
}

fn charge_payment() {
    ctx_info!("Charging payment method");
    ctx_info!("Payment charged successfully");
}

fn send_receipt() {
    ctx_info!("Sending payment receipt");
    ctx_info!("Payment receipt sent");
}

/// Example 6: Async function with comprehensive logging.
#[params(
    span,
    custom(
        service = "notification-service",
        version = "1.5.2",
        async_enabled = true
    ),
    fields(
        user.id,
        user.profile.preferences.notifications_enabled,
        notification_type,
        priority
    )
)]
async fn send_batch_notifications(
    user: User,
    notification_type: String,
    priority: u8,
    _templates: Vec<String>,  // Not logged - could be large
    _api_credentials: ApiKey, // Not logged - sensitive
) {
    info!("Starting batch notification processing");

    // Async child functions inherit span context
    prepare_notification_batch().await;
    deliver_notifications().await;

    info!("Batch notification processing completed");
}

// Async child functions
async fn prepare_notification_batch() {
    ctx_info!("Preparing notification batch");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    ctx_info!("Notification batch prepared");
}

async fn deliver_notifications() {
    ctx_info!("Delivering notifications");

    // Simulate async delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    ctx_info!("Notifications delivered");
}

/// Example 7: Error handling with comprehensive logging.
#[params(
    fields(
        operation_id,
        user.id,
        retry_count,
        max_retries
    ),
    custom(
        service = "data-processor",
        error_handling = "comprehensive"
    )
)]
fn process_data_with_error_handling(
    operation_id: String,
    user: User,
    _data: Vec<u8>, // Not logged - could be large binary data
    retry_count: u32,
    max_retries: u32,
) -> Result<String, String> {
    info!("Starting data processing with error handling");

    if retry_count > max_retries {
        error!("Maximum retry count exceeded");
        return Err("Max retries exceeded".to_string());
    }

    // Simulate processing
    if operation_id.starts_with("fail") {
        error!("Simulated processing failure");
        return Err("Processing failed".to_string());
    }

    info!("Data processing completed successfully");
    Ok("success".to_string())
}

/// Example 8: Method in impl block with all features.
struct UserService {
    _service_config: ServiceConfig,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ServiceConfig {
    max_concurrent_requests: u32,
    timeout_seconds: u32,
    cache_enabled: bool,
}

impl UserService {
    /// Comprehensive user management operation with all macro features.
    #[params(
        span,
        custom(
            service = "user-service",
            version = "4.2.1",
            method = "manage_user_lifecycle"
        ),
        fields(
            user.id,
            user.username,
            operation_type,
            self._service_config.cache_enabled,
            self._service_config.timeout_seconds
        )
    )]
    fn manage_user_lifecycle(
        &self,
        user: User,
        operation_type: String,
        _metadata: HashMap<String, String>, // Not logged - could contain sensitive data
    ) -> Result<String, String> {
        info!("Starting user lifecycle management");

        // Method calls that inherit span context
        self.validate_user_state()?;
        self.update_user_metadata()?;

        info!("User lifecycle management completed");
        Ok("lifecycle_managed".to_string())
    }

    fn validate_user_state(&self) -> Result<(), String> {
        ctx_info!("Validating user state");
        ctx_info!("User state validation completed");
        Ok(())
    }

    fn update_user_metadata(&self) -> Result<(), String> {
        ctx_info!("Updating user metadata");
        ctx_info!("User metadata updated");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Initialize comprehensive tracing subscriber
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("=== Comprehensive log_args Features Demonstration ===");
    println!("This example showcases all macro features in real-world scenarios.");
    println!();

    // Create comprehensive test data
    let user = User {
        id: 12345,
        username: "alice_johnson".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "$2b$12$secrethash123456789".to_string(),
        profile: UserProfile {
            first_name: "Alice".to_string(),
            last_name: "Johnson".to_string(),
            preferences: UserPreferences {
                theme: "dark".to_string(),
                language: "en".to_string(),
                notifications_enabled: true,
            },
        },
    };

    let api_key = ApiKey {
        key_id: "key_123".to_string(),
        secret: "sk_live_secret123456789".to_string(),
        permissions: vec!["admin".to_string()],
    };

    // Example 1: Basic parameter logging
    let validation_rules = vec!["min_length:3".to_string(), "max_length:50".to_string()];
    basic_user_validation(user.clone(), validation_rules);

    println!();

    // Example 2: Selective field logging
    update_user_profile(
        user.clone(),
        "newemail@example.com".to_string(),
        "new_secure_password".to_string(),
        "profile_update".to_string(),
        false,
        api_key.clone(),
    );

    println!();

    // Example 3: Custom fields for metadata
    authenticate_user_with_metadata(
        "alice_johnson".to_string(),
        "$2b$12$hashed_password_secure".to_string(),
        true,
    );

    println!();

    // Example 4: Span context propagation
    let operation = UserOperation {
        operation_type: "data_migration".to_string(),
        priority: 1,
        metadata: {
            let mut map = HashMap::new();
            map.insert("source".to_string(), "legacy_system".to_string());
            map
        },
    };
    process_user_operation(
        user.clone(),
        operation,
        "sensitive_migration_token".to_string(),
    );

    println!();

    // Example 5: Combined attributes (span + custom + fields)
    process_payment_transaction(
        "txn_789123456".to_string(),
        99.99,
        "USD".to_string(),
        user.clone(),
        "4242424242424242".to_string(),
        true,
        "internal_processing_key_secret".to_string(),
    );

    println!();

    // Example 6: Async function with comprehensive logging
    let templates = vec!["welcome_template".to_string()];
    send_batch_notifications(user.clone(), "email".to_string(), 1, templates, api_key).await;

    println!();

    // Example 7: Error handling
    let data = vec![1, 2, 3, 4, 5];
    match process_data_with_error_handling(
        "success_operation_123".to_string(),
        user.clone(),
        data,
        0,
        3,
    ) {
        Ok(result) => info!("Data processing result: {}", result),
        Err(error) => error!("Data processing error: {}", error),
    }

    println!();

    // Example 7b: Error case
    match process_data_with_error_handling(
        "fail_operation_456".to_string(),
        user.clone(),
        vec![1, 2, 3],
        0,
        3,
    ) {
        Ok(result) => info!("Data processing result: {}", result),
        Err(error) => error!("Expected error occurred: {}", error),
    }

    println!();

    // Example 8: Method with comprehensive features
    let service_config = ServiceConfig {
        max_concurrent_requests: 100,
        timeout_seconds: 30,
        cache_enabled: true,
    };
    let user_service = UserService {
        _service_config: service_config,
    };

    let mut metadata = HashMap::new();
    metadata.insert("operation_source".to_string(), "admin_panel".to_string());

    match user_service.manage_user_lifecycle(user, "lifecycle_update".to_string(), metadata) {
        Ok(result) => info!("User lifecycle management result: {}", result),
        Err(error) => error!("User lifecycle management error: {}", error),
    }

    println!();
    println!("=== Summary of Features Demonstrated ===");
    println!("✅ Basic parameter logging (#[params])");
    println!("✅ Selective field logging (#[params(fields(...))])");
    println!("✅ Custom static fields (#[params(custom(...))])");
    println!("✅ Span context propagation (#[params(span)])");
    println!("✅ Combined attributes (span + custom + fields)");
    println!("✅ Async function support");
    println!("✅ Error handling and Result types");
    println!("✅ Method support in impl blocks");
    println!("✅ Security-conscious selective logging");
    println!();
    println!("🎯 All log_args features successfully demonstrated!");
    println!("📊 Check the JSON logs above to see the structured output.");
    println!("🔒 Notice how sensitive data is excluded from logs.");
    println!("🔗 Observe context propagation in span-enabled functions.");
}
