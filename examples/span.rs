use log_args::params;
use tracing::info;
use log_args_runtime::__PARENT_LOG_ARGS;
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
    debug!("Inside my_function");
    info!("Inside my_function");
    warn!("Inside my_function");
    error!("Inside my_function");
    sub_function();
}

#[params]
fn sub_function() {
    debug!("Inside sub_function");
    info!("Inside sub_function");
    warn!("Inside sub_function");
    error!("Inside sub_function");
}

#[params]
fn my_function2(arg1: i32) {
    debug!("Inside my_function2");
    info!("Inside my_function2");
    warn!("Inside my_function2");
    error!("Inside my_function2");
    sub_function();
}
