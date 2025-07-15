use log_args::params;
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
#[params]
fn my_handler_all(_user: User, _config: Config) {
    info!("Handler invoked");
}

// Log specific arguments
#[params(fields(user.id, config.debug))]
fn my_handler_fields(user: User, config: Config) {
    info!("Fields filtered");
}

// Log subfields
#[params(fields(user.id, user.name))]
fn my_handler_subfields(user: User) {
    warn!("User processing failed");
}

// Log with custom key-value pairs
#[params(fields(user.id), custom(service = "auth", env = "prod"))]
fn login(user: User) {
    info!("Login attempt started");
    debug!("test debug logging also");
    error!("test error logging also");
}

// Log from async functions
#[params(fields(user.id))]
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
