use log_args::params;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
struct User { id: u32, name: String }

#[params(fields(user.id))]
#[allow(dead_code, unused_variables)]
async fn process(user: User) {
    info!("Processing");
}

fn main() {}
