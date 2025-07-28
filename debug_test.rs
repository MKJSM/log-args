use log_args::params;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, Registry};

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
        self.buf.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.lock().unwrap().flush()
    }
}

fn main() {
    let writer = MockWriter::new();
    let writer_clone = writer.clone();
    let subscriber = Registry::default().with(
        fmt::layer()
            .json()
            .with_writer(move || writer_clone.clone()),
    );

    tracing::subscriber::with_default(subscriber, || {
        #[params(custom(service = "test-service", version = "1.0.0", debug = true))]
        fn test_function(data: String) {
            info!("Custom fields test");
        }

        test_function("test_data".to_string());
    });

    let logs = writer.get_logs();
    println!("Raw logs: {}", logs);
    
    let log_json: Value = serde_json::from_str(&logs).expect("Failed to parse log as JSON");
    println!("Parsed JSON: {:#}", log_json);
    
    println!("Fields:");
    if let Some(fields) = log_json["fields"].as_object() {
        for (key, value) in fields {
            println!("  {}: {:?}", key, value);
        }
    }
}
