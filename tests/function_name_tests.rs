//! Unit tests for function name logging feature
//!
//! These tests verify that function names are correctly included in log output
//! when the appropriate Cargo features are enabled.

use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, prelude::*, Registry};

/// Mock writer for capturing log output in tests
#[derive(Clone)]
struct MockWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl MockWriter {
    fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> String {
        let mut buf = self.buf.lock().unwrap();
        let output = String::from_utf8_lossy(&buf).to_string();
        buf.clear();
        output
    }
}

impl std::io::Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MockWriter {
    type Writer = MockWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[derive(Debug, Clone)]
struct TestUser {
    id: u64,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_tracing() -> MockWriter {
        let mock_writer = MockWriter::new();
        let subscriber =
            Registry::default().with(fmt::layer().json().with_writer(mock_writer.clone()));
        tracing::subscriber::set_global_default(subscriber).ok();
        mock_writer
    }

    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    #[test]
    fn test_basic_function_name_logging() {
        let mock_writer = setup_tracing();

        #[params]
        fn test_function(username: String, _password: String) {
            info!("User authentication attempt");
        }

        test_function("alice".to_string(), "secret".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        assert!(!log_lines.is_empty(), "Should have log output");

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is included
        assert!(
            log_json["fields"]["function"].is_string(),
            "Should include function name"
        );

        // Verify parameters are logged
        assert_eq!(
            log_json["fields"]["username"].as_str().unwrap(),
            "\"alice\""
        );
        assert_eq!(
            log_json["fields"]["_password"].as_str().unwrap(),
            "\"secret\""
        );
    }

    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    #[test]
    fn test_function_name_with_selective_fields() {
        let mock_writer = setup_tracing();

        #[params(fields(user.id, user.name))]
        fn process_user_data(user: TestUser, _api_key: String) {
            info!("Processing user data");
        }

        let user = TestUser {
            id: 12345,
            name: "Alice".to_string(),
        };

        process_user_data(user, "secret_key".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is included
        assert!(log_json["fields"]["function"].is_string());

        // Verify only selected fields are logged
        assert_eq!(log_json["fields"]["user.id"].as_str().unwrap(), "12345");
        assert_eq!(
            log_json["fields"]["user.name"].as_str().unwrap(),
            "\"Alice\""
        );

        // Verify api_key is not logged (not in fields list)
        assert!(log_json["fields"]["_api_key"].is_null());
    }

    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    #[test]
    fn test_function_name_with_custom_fields() {
        let mock_writer = setup_tracing();

        #[params(custom(service = "auth", version = "2.0"))]
        fn validate_credentials(_username: String, _password: String) {
            info!("Validating user credentials");
        }

        validate_credentials("alice".to_string(), "password".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is included
        assert!(log_json["fields"]["function"].is_string());

        // Verify custom fields are included
        assert_eq!(log_json["fields"]["service"].as_str().unwrap(), "auth");
        assert_eq!(log_json["fields"]["version"].as_str().unwrap(), "2.0");
    }

    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    #[test]
    fn test_function_name_with_span_propagation() {
        let mock_writer = setup_tracing();

        #[params(span, fields(user_id))]
        fn parent_function(user_id: u64, _sensitive_data: String) {
            info!("Parent function started");
            child_function();
            info!("Parent function completed");
        }

        fn child_function() {
            ctx_info!("Child function called");
        }

        parent_function(12345, "sensitive".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        assert!(log_lines.len() >= 3, "Should have multiple log entries");

        // Check parent function logs
        let parent_log: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");
        assert!(parent_log["fields"]["function"].is_string());
        assert_eq!(parent_log["fields"]["user_id"].as_str().unwrap(), "12345");

        // Check child function inherits context
        let child_log: Value = serde_json::from_str(log_lines[1]).expect("Should be valid JSON");
        assert!(child_log["fields"]["context"]
            .as_str()
            .unwrap()
            .contains("user_id=12345"));
        assert!(child_log["fields"]["context"]
            .as_str()
            .unwrap()
            .contains("function="));
    }

    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    #[tokio::test]
    async fn test_async_function_name_logging() {
        let mock_writer = setup_tracing();

        #[params(fields(user_id, operation))]
        async fn async_operation(user_id: u64, operation: String, _credentials: String) {
            info!("Starting async operation");
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            info!("Async operation completed");
        }

        async_operation(12345, "data_export".to_string(), "secret".to_string()).await;

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        assert!(
            log_lines.len() >= 2,
            "Should have multiple async log entries"
        );

        for log_line in log_lines {
            let log_json: Value = serde_json::from_str(log_line).expect("Should be valid JSON");

            // Verify function name is included in async logs
            assert!(log_json["fields"]["function"].is_string());
            assert_eq!(log_json["fields"]["user_id"].as_str().unwrap(), "12345");
            assert_eq!(
                log_json["fields"]["operation"].as_str().unwrap(),
                "\"data_export\""
            );
        }
    }

    #[cfg(feature = "function-names-pascal")]
    #[test]
    fn test_pascal_case_function_names() {
        let mock_writer = setup_tracing();

        #[params]
        fn snake_case_function_name(_param: String) {
            info!("Testing PascalCase conversion");
        }

        snake_case_function_name("test".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is converted to PascalCase
        assert_eq!(
            log_json["fields"]["function"].as_str().unwrap(),
            "SnakeCaseFunctionName"
        );
    }

    #[cfg(feature = "function-names-snake")]
    #[test]
    fn test_snake_case_function_names() {
        let mock_writer = setup_tracing();

        #[params]
        fn snake_case_function_name(_param: String) {
            info!("Testing snake_case preservation");
        }

        snake_case_function_name("test".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is preserved as snake_case
        assert_eq!(
            log_json["fields"]["function"].as_str().unwrap(),
            "snake_case_function_name"
        );
    }

    #[cfg(feature = "function-names-camel")]
    #[test]
    fn test_camel_case_function_names() {
        let mock_writer = setup_tracing();

        #[params]
        fn snake_case_function_name(_param: String) {
            info!("Testing camelCase conversion");
        }

        snake_case_function_name("test".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify function name is converted to camelCase
        assert_eq!(
            log_json["fields"]["function"].as_str().unwrap(),
            "snakeCaseFunctionName"
        );
    }
}
