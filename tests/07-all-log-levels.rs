use log_args::params;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
}

#[params(fields(user.id, user.name))]
#[allow(dead_code, unused_variables)]
fn process(user: User) {
    info!("Info log");
    warn!("Warn log");
    error!("Error log");
    debug!("Debug log");
}

fn main() {}
