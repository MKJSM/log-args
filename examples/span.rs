use log_args::params;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    my_function(123);
    my_function2(123);
}

#[params(span = true)]
fn my_function(arg1: i32) {
    tracing::debug!("Inside my_function");
    tracing::info!("Inside my_function");
    warn!("Inside my_function");
    error!("Inside my_function");
    sub_function();
}

#[params]
fn sub_function() {
    tracing::debug!("Inside sub_function");
    tracing::info!("Inside sub_function");
    warn!("Inside sub_function");
    error!("Inside sub_function");
}

#[params]
fn my_function2(arg1: i32) {
    tracing::debug!("Inside my_function2");
    tracing::info!("Inside my_function2");
    warn!("Inside my_function2");
    error!("Inside my_function2");
    sub_function();
}
