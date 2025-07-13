use log_args::params;
use tracing::info;
use tracing_subscriber;

#[test]
fn test_span_feature() {
    let _ = tracing_subscriber::fmt::try_init();

    #[params(span = true)]
    fn test_function(arg1: i32, arg2: &str) {
        info!("Inside test_function");
    }

    test_function(42, "hello");
}

#[test]
fn test_span_feature_with_fields() {
    let _ = tracing_subscriber::fmt::try_init();

    #[params(fields(arg1), span = true)]
    fn test_function_with_fields(arg1: i32, arg2: &str) {
        info!("Inside test_function_with_fields");
    }

    test_function_with_fields(100, "world");
}
