use log_args::params;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

// This should fail because `fields` is misspelled
#[params(field(user.id))]
fn process(user: User) {}

fn main() {}
