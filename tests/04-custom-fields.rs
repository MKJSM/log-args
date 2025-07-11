use log_args::log_args;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
struct User { id: u32, name: String }

#[log_args(fields(user.id), custom(service = "test"))]
#[allow(dead_code, unused_variables)]
fn process(user: User) {
    info!("Processing");
}

fn main() {}
