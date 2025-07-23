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

// Log all arguments: User and Config will be Debug formatted by default
#[params]
fn my_handler_all(_user: User, _config: Config) {
    info!("Handler invoked");
}

// Log specific arguments: user.id (u32) and config.debug (bool) will be Debug formatted by default
#[params(fields(user.id, config.debug))]
fn my_handler_fields(user: User, config: Config) {
    info!("Fields filtered");
}

// Log subfields: user.id will be Debug, user.name will be Display
// This ensures "Alice" shows as Alice, not "Alice"
#[params(fields(user.id, user.name), display_fields(user.name))]
fn my_handler_subfields(user: User) {
    warn!("User processing failed");
}

// Log with custom key-value pairs: user.id will be Debug, custom fields are Display
#[params(fields(user.id), custom(service = "auth", env = "prod"))]
fn login(user: User) {
    info!("Login attempt started");
    debug!("test debug logging also");
    error!("test error logging also");
}

// Log from async functions: user.id will be Debug
#[params(fields(user.id))]
async fn send_email(user: User) {
    info!("Email triggered");
}

// Demonstrate `clone_upfront` with `display_fields`
// user.id will be Debug, user.name will be Display
#[params(clone_upfront, fields(user.id, user.name), display_fields(user.name))]
async fn process_user_async_cloned(user: User) {
    info!("Processing user with clone_upfront");
    debug!("Another debug message within async function");
}

fn main() {
    // Initialize tracing subscriber to see debug level logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("\n--- Calling my_handler_all ---");
    let user_all = User {
        id: 1,
        name: "Alice".to_string(),
    };
    let config_all = Config { debug: true };
    my_handler_all(user_all, config_all);
    // Expected: user={id: 1, name: "Alice"}, config={debug: true} (Debug format for structs)

    println!("\n--- Calling my_handler_fields ---");
    let user_fields = User {
        id: 2,
        name: "Bob".to_string(),
    };
    let config_fields = Config { debug: false };
    my_handler_fields(user_fields, config_fields);
    // Expected: user.id=2, config.debug=false (Debug format for primitives)

    println!("\n--- Calling my_handler_subfields ---");
    let user_subfields = User {
        id: 3,
        name: "Charlie".to_string(),
    };
    my_handler_subfields(user_subfields);
    // Expected: user.id=3 (Debug), user.name=Charlie (Display, no quotes for String)

    println!("\n--- Calling login ---");
    let user_login = User {
        id: 4,
        name: "David".to_string(),
    };
    login(user_login);
    // Expected: user.id=4, service=auth, env=prod (custom fields are Display)

    println!("\n--- Calling send_email (async) ---");
    let user_email = User {
        id: 5,
        name: "Eve".to_string(),
    };
    futures::executor::block_on(send_email(user_email));
    // Expected: user.id=5

    println!(
        "\n--- Calling process_user_async_cloned (async with clone_upfront and display_fields) ---"
    );
    let user_cloned_async = User {
        id: 6,
        name: "Frank".to_string(),
    };
    futures::executor::block_on(process_user_async_cloned(user_cloned_async));
    // Expected: user.id=6 (Debug), user.name=Frank (Display, no quotes for String)
}
