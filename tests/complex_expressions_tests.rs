//! Unit tests for complex field expressions
//!
//! These tests verify that the macro correctly handles nested fields,
//! method calls, and complex expressions in field specifications.

use log_args::params;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
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
        let buf = self.buf.lock().unwrap();
        let output = String::from_utf8_lossy(&buf).to_string();
        output
    }

    fn clear_logs(&self) {
        self.buf.lock().unwrap().clear();
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
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Debug, Clone)]
struct Contact {
    name: String,
    email: String,
    phone: String,
    addresses: Vec<Address>,
}

#[derive(Debug, Clone)]
struct Person {
    id: u64,
    name: String,
    contact: Contact,
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct Organization {
    name: String,
    people: Vec<Person>,
    metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_tracing() -> (MockWriter, tracing::subscriber::DefaultGuard) {
        let mock_writer = MockWriter::new();
        let subscriber =
            Registry::default().with(fmt::layer().json().with_writer(mock_writer.clone()));
        let guard = tracing::subscriber::set_default(subscriber);
        (mock_writer, guard)
    }

    #[test]
    fn test_nested_field_access() {
        let (mock_writer, _guard) = setup_tracing();

        #[params(fields(person.id, person.name, person.contact.name, person.contact.email))]
        fn process_person(person: Person, _sensitive_data: String) {
            info!("Processing person data");
        }

        let person = Person {
            id: 12345,
            name: "Alice Johnson".to_string(),
            contact: Contact {
                name: "Alice J.".to_string(),
                email: "alice@example.com".to_string(),
                phone: "555-0123".to_string(),
                addresses: vec![],
            },
            tags: vec!["vip".to_string()],
        };

        process_person(person, "secret".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify nested fields are correctly logged
        assert_eq!(log_json["fields"]["person.id"].as_str().unwrap(), "12345");
        assert_eq!(
            log_json["fields"]["person.name"].as_str().unwrap(),
            "\"Alice Johnson\""
        );
        assert_eq!(
            log_json["fields"]["person.contact.name"].as_str().unwrap(),
            "\"Alice J.\""
        );
        assert_eq!(
            log_json["fields"]["person.contact.email"].as_str().unwrap(),
            "\"alice@example.com\""
        );

        // Verify sensitive data is not logged
        assert!(log_json["fields"]["_sensitive_data"].is_null());
    }

    #[test]
    fn test_method_call_expressions() {
        let (mock_writer, _guard) = setup_tracing();

        #[params(fields(data.len(), text.to_uppercase(), tags.is_empty()))]
        fn process_with_methods(data: Vec<u8>, text: String, tags: Vec<String>, _secret: String) {
            info!("Processing with method calls");
        }

        let data = vec![1, 2, 3, 4, 5];
        let text = "hello world".to_string();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];

        process_with_methods(data, text, tags, "secret".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify method call results are logged
        assert_eq!(log_json["fields"]["data.len()"].as_str().unwrap(), "5");
        assert_eq!(
            log_json["fields"]["text.to_uppercase()"].as_str().unwrap(),
            "\"HELLO WORLD\""
        );
        assert_eq!(
            log_json["fields"]["tags.is_empty()"].as_str().unwrap(),
            "false"
        );
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let (mock_writer, _guard) = setup_tracing();

        #[params(fields(
            org.people.len(),
            org.people[0].contact.addresses.len(),
            org.metadata.get("department").unwrap_or(&"unknown".to_string())
        ))]
        fn process_organization(org: Organization, _api_key: String) {
            info!("Processing organization");
        }

        let mut metadata = HashMap::new();
        metadata.insert("department".to_string(), "engineering".to_string());

        let org = Organization {
            name: "Tech Corp".to_string(),
            people: vec![Person {
                id: 1,
                name: "Alice".to_string(),
                contact: Contact {
                    name: "Alice".to_string(),
                    email: "alice@techcorp.com".to_string(),
                    phone: "555-0123".to_string(),
                    addresses: vec![Address {
                        street: "123 Main St".to_string(),
                        city: "Tech City".to_string(),
                        zip: "12345".to_string(),
                    }],
                },
                tags: vec![],
            }],
            metadata,
        };

        process_organization(org, "secret_key".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs
            .trim()
            .split('\n')
            .filter(|line| !line.is_empty())
            .collect();

        assert!(
            !log_lines.is_empty(),
            "Expected at least one log line, got: {logs}"
        );
        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify deeply nested expressions work
        assert_eq!(
            log_json["fields"]["org.people.len()"].as_str().unwrap(),
            "1"
        );
        assert_eq!(
            log_json["fields"]["org.people [0].contact.addresses.len()"]
                .as_str()
                .unwrap(),
            "1"
        );
        assert_eq!(
            log_json["fields"]
                ["org.metadata.get(\"department\").unwrap_or(& \"unknown\".to_string())"]
                .as_str()
                .unwrap(),
            "\"engineering\""
        );
    }

    #[test]
    fn test_complex_expressions_with_custom_fields() {
        let (mock_writer, _guard) = setup_tracing();

        #[params(
            fields(person.contact.addresses.len(), person.tags.join(",")),
            custom(service = "user-processor", version = "1.0")
        )]
        fn process_user_profile(person: Person, _password: String) {
            info!("Processing user profile");
        }

        let person = Person {
            id: 123,
            name: "Bob".to_string(),
            contact: Contact {
                name: "Bob Smith".to_string(),
                email: "bob@example.com".to_string(),
                phone: "555-9876".to_string(),
                addresses: vec![
                    Address {
                        street: "456 Oak Ave".to_string(),
                        city: "Springfield".to_string(),
                        zip: "67890".to_string(),
                    },
                    Address {
                        street: "789 Pine St".to_string(),
                        city: "Riverside".to_string(),
                        zip: "54321".to_string(),
                    },
                ],
            },
            tags: vec!["premium".to_string(), "verified".to_string()],
        };

        process_user_profile(person, "secret_password".to_string());

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs.trim().split('\n').collect();

        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify complex expressions with custom fields
        assert_eq!(
            log_json["fields"]["person.contact.addresses.len()"]
                .as_str()
                .unwrap(),
            "2"
        );
        assert_eq!(
            log_json["fields"]["person.tags.join(\",\")"]
                .as_str()
                .unwrap(),
            "\"premium,verified\""
        );

        // Verify custom fields are included
        assert_eq!(
            log_json["fields"]["service"].as_str().unwrap(),
            "user-processor"
        );
        assert_eq!(log_json["fields"]["version"].as_str().unwrap(), "1.0");

        // Verify password is not logged
        assert!(log_json["fields"]["_password"].is_null());
    }

    #[test]
    fn test_expressions_with_error_handling() {
        let (mock_writer, _guard) = setup_tracing();

        #[params(fields(
            data.get(0).unwrap_or(&0),
            text.chars().count(),
            optional_value.as_ref().map(|v| v.len()).unwrap_or(0)
        ))]
        fn safe_processing(
            data: Vec<i32>,
            text: String,
            optional_value: Option<String>,
            _secret: String,
        ) {
            info!("Safe processing with error handling");
        }

        safe_processing(
            vec![42, 84, 126],
            "test string".to_string(),
            Some("optional data".to_string()),
            "secret".to_string(),
        );

        let logs = mock_writer.get_logs();
        let log_lines: Vec<&str> = logs
            .trim()
            .split('\n')
            .filter(|line| !line.is_empty())
            .collect();

        assert!(
            !log_lines.is_empty(),
            "Expected at least one log line, got: {logs}"
        );
        let log_json: Value = serde_json::from_str(log_lines[0]).expect("Should be valid JSON");

        // Verify error-safe expressions work
        assert_eq!(
            log_json["fields"]["data.get(0).unwrap_or(& 0)"]
                .as_str()
                .unwrap(),
            "42"
        );
        assert_eq!(
            log_json["fields"]["text.chars().count()"].as_str().unwrap(),
            "11"
        );
        assert_eq!(
            log_json["fields"]["optional_value.as_ref().map(| v | v.len()).unwrap_or(0)"]
                .as_str()
                .unwrap(),
            "13"
        );
    }
}
