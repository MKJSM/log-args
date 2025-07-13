use log_args::params;
use tracing::info;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt().init();
    my_function(123);
    my_function2(123);
}

#[params(span = true)]
fn my_function(arg1: i32) {
    info!("Inside my_function");
    sub_function();
}

#[params]
fn sub_function() {
    info!("Inside sub_function");
}

#[params]
fn my_function2(arg1: i32) {
    info!("Inside my_function");
    sub_function();
}
