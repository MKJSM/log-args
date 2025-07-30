use log_args::params;
use tracing::info;

/// Parent function that sets context
pub fn process_payment(user_id: String, currency: String, amount: String) {
    info!("Payment processed successfully");

    // Call child function
    validate_payment();
}

/// Child function that should inherit context automatically
pub fn validate_payment() {
    info!("Payment validation completed");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).json().init();

    process_payment("789".to_string(), "USD".to_string(), "99.99".to_string());
}
