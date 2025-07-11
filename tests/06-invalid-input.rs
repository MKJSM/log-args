use log_args::log_args;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

// This should fail because `fields` is misspelled
#[log_args(field(user.id))]
fn process(user: User) {}

fn main() {}
