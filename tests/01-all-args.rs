use log_args::params;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
}

#[params(fields(user.id, user.name, value))]
#[allow(dead_code, unused_variables)]
fn process(user: User, value: i32) {
    info!("Processing");
}

fn main() {}
