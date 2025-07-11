use log_args::log_args;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
}

#[log_args]
#[allow(dead_code, unused_variables)]
fn process(user: User, value: i32) {
    info!("Processing");
}

fn main() {}
