use log_args::params;
use tracing::info;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
}

#[params]
#[allow(dead_code, unused_variables)]
fn process(user: User, value: i32) {
    info!("Processing");
}

fn main() {}
