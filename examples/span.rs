use log_args::params;
use tracing::debug;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    my_function(123);
    my_function2(123);
    my_function3(123);
}

#[params(span)]
fn my_function(arg1: i32) {
    debug!("Inside my_function");
    sub_function();
}

#[params]
fn sub_function() {
    debug!("Inside sub_function");
}

#[params]
fn my_function2(_arg1: i32) {
    debug!("Inside my_function2");
    sub_function();
}

#[params]
fn sub_function2(_name: String) {
    debug!("Inside sub_function2");
}

#[params(span)]
fn my_function3(arg1: i32) {
    debug!("Inside my_function3");
    sub_function2("name".to_string());
}
