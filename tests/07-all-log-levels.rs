use log_args::log_args;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
}

#[log_args]
#[allow(dead_code, unused_variables)]
fn process(user: User) {
    info!("Info log");
    warn!("Warn log");
    error!("Error log");
    debug!("Debug log");
}

fn main() {}
