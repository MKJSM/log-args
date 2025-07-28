//! # Selective Field Logging Example
//!
//! This example demonstrates the `fields` attribute for selective parameter logging.
//! The `fields` attribute allows you to specify exactly which function parameters
//! or parameter fields should be logged, instead of logging all parameters.
//!
//! ## Usage
//! ```bash
//! cargo run --example fields
//! ```
//!
//! ## Features Demonstrated
//! - Selective logging of specific fields
//! - Excluding sensitive data from logs
//! - Deep nested field access
//! - Method calls and expressions in field selection
//! - Security and compliance benefits

use log_args::params;
use std::collections::HashMap;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String, // Sensitive - should not be logged
    profile: UserProfile,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserProfile {
    first_name: String,
    last_name: String,
    age: u32,
    preferences: UserPreferences,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserPreferences {
    theme: String,
    language: String,
    notifications_enabled: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ApiKey {
    key_id: String,
    secret: String, // Sensitive - should not be logged
    permissions: Vec<String>,
}

/// Authenticates user with selective field logging.
/// Only logs safe user fields, excludes sensitive API key data.
#[params(fields(user.id, user.username, user.email, remember_me))]
fn authenticate_user(user: User, _api_key: ApiKey, remember_me: bool) {
    info!("Starting user authentication");
    info!("User authentication completed");
}

/// Updates user profile with deep nested field access.
/// Logs specific profile fields while excluding sensitive password.
#[params(fields(
    user.id,
    user.profile.first_name,
    user.profile.last_name,
    user.profile.preferences.theme,
    user.profile.preferences.language
))]
fn update_user_profile(user: User, _new_email: String, _password: String) {
    info!("Updating user profile");
    info!("User profile updated successfully");
}

/// Processes user batch with method calls and expressions.
/// Demonstrates logging collection length and selective config fields.
#[params(fields(
    users.len(),
    batch_size,
    max_retries
))]
fn process_user_batch(users: Vec<User>, batch_size: usize, max_retries: u32) {
    info!("Starting batch user processing");
    info!("Batch processing completed");
}

/// Sends notifications with selective async logging.
#[params(fields(user_id, notification_type, priority))]
async fn send_user_notification(
    user_id: u64,
    notification_type: String,
    _message: String, // Not logged - could contain sensitive info
    priority: String,
) {
    info!("Preparing user notification");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info!("User notification sent successfully");
}

/// Processes complex data with advanced expressions.
/// Shows complex field expressions and method calls.
#[params(fields(
    user.profile.preferences.notifications_enabled,
    settings.len(),
    tags.len()
))]
fn process_complex_data(
    user: User,
    settings: HashMap<String, String>,
    tags: Vec<String>,
    _sensitive_token: String, // Not logged
) {
    info!("Processing complex data structure");
    info!("Complex data processing completed");
}

/// User service for demonstrating method selective logging.
struct UserService {
    _database_url: String,
    _api_secret: String, // Sensitive field
}

impl UserService {
    /// Gets user data with selective field logging.
    #[params(fields(user_id, include_sensitive))]
    fn get_user_data(&self, user_id: u64, include_sensitive: bool, _admin_token: String) {
        info!("Fetching user data");
        info!("User data retrieved successfully");
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for JSON output
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("=== Selective Field Logging Example ===");
    println!("This example demonstrates selective logging of specific fields only.");
    println!();

    // Create test data
    let user = User {
        id: 12345,
        username: "alice_johnson".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "$2b$12$secrethash123456789".to_string(), // Sensitive
        profile: UserProfile {
            first_name: "Alice".to_string(),
            last_name: "Johnson".to_string(),
            age: 28,
            preferences: UserPreferences {
                theme: "dark".to_string(),
                language: "en".to_string(),
                notifications_enabled: true,
            },
        },
    };

    let api_key = ApiKey {
        key_id: "key_123".to_string(),
        secret: "sk_live_secret123456789".to_string(), // Sensitive
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    // Example 1: Basic field selection (excluding sensitive data)
    authenticate_user(user.clone(), api_key, true);

    println!();

    // Example 2: Deep nested field access
    update_user_profile(
        user.clone(),
        "newemail@example.com".to_string(),
        "new_password".to_string(),
    );

    println!();

    // Example 3: Selective logging with expressions
    let users_batch = vec![user.clone(), user.clone()];
    process_user_batch(users_batch, 10, 3);

    println!();

    // Example 4: Async function with selective fields
    send_user_notification(
        67890,
        "welcome".to_string(),
        "Welcome to our service! Your account is ready.".to_string(),
        "high".to_string(),
    )
    .await;

    println!();

    // Example 5: Complex nested expressions
    let mut settings = HashMap::new();
    settings.insert("theme".to_string(), "dark".to_string());
    settings.insert("lang".to_string(), "en".to_string());

    let tags = vec![
        "urgent-notification".to_string(),
        "user-update".to_string(),
        "urgent-security".to_string(),
    ];

    process_complex_data(
        user,
        settings,
        tags,
        "sensitive_admin_token_xyz789".to_string(),
    );

    println!();

    // Example 6: Method with selective logging
    let service = UserService {
        _database_url: "postgresql://localhost:5432/users".to_string(),
        _api_secret: "super_secret_api_key_abc123".to_string(), // Sensitive
    };
    service.get_user_data(12345, false, "admin_token_xyz".to_string());

    println!();
    println!("✅ Check the JSON logs above to see only the specified fields are logged!");
    println!("🔒 Notice how sensitive data (passwords, tokens, secrets) is excluded.");
}
