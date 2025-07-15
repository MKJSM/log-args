use log_args::log_args;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Config {
    debug: bool,
}

// Log all arguments
#[log_args]
fn my_handler_all(user: User, config: Config) {
    info!("Handler invoked");
}

// Log specific arguments
#[log_args(fields(user, config))]
fn my_handler_fields(user: User, config: Config) {
    info!("Fields filtered");
}

// Log subfields
#[log_args(fields(user.id, user.name))]
fn my_handler_subfields(user: User) {
    warn!("User processing failed");
}

// Log with custom key-value pairs
#[log_args(fields(user.id), custom(service = "auth", env = "prod"))]
fn login(user: User) {
    info!("Login attempt started");
    debug!("test debug logging also");
    error!("test error logging also");
}

// Log from async functions
#[log_args(fields(user.id))]
async fn send_email(user: User) {
    info!("Email triggered");
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    let config = Config { debug: true };
    my_handler_all(user, config);

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    let config = Config { debug: true };
    my_handler_fields(user, config);

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    my_handler_subfields(user);

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    login(user);

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    // We need a runtime for the async function
    futures::executor::block_on(send_email(user));
}
