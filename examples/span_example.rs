use log_args::params;
use tracing::{debug, info, warn, error};
use tracing_subscriber;
use std::io::{self, Write};

// Import for nested spans demonstration
use tracing::Instrument;
use futures::FutureExt;

// Import for custom formatting
use tracing_subscriber::fmt::format::FmtSpan;
use serde_json::{self, json, Value};
use serde::Serialize;

// Custom JSON formatter for clean output
#[derive(Debug, Serialize)]
struct CustomJsonEvent {
    timestamp: String,
    level: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    fields: std::collections::HashMap<String, Value>,
}

/// Wraps stdout to process JSON before output
#[derive(Debug, Clone)]
struct JsonProcessor {}

impl Write for JsonProcessor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let input_len = buf.len();
        
        // Convert buffer to string and process
        if let Ok(json_str) = std::str::from_utf8(buf) {
            // Parse the JSON
            if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                if let Some(obj) = value.as_object() {
                    // Extract the core fields
                    let timestamp = obj.get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string();
                        
                    let level = obj.get("level")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string();
                        
                    let target = obj.get("target")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string();
                    
                    let message = obj.get("fields")
                        .and_then(|f| f.as_object())
                        .and_then(|f| f.get("message"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string());
                    
                    // Extract and process fields
                    let mut fields = std::collections::HashMap::new();
                    
                    if let Some(field_obj) = obj.get("fields").and_then(|f| f.as_object()) {
                        for (k, v) in field_obj {
                            if k == "message" { continue; } // Message is handled separately
                            
                            if k == "user.id" {
                                // Convert user.id to a number if it's a string containing digits
                                if let Some(id_str) = v.as_str() {
                                    if let Ok(id) = id_str.parse::<u32>() {
                                        fields.insert(k.clone(), json!(id));
                                        continue;
                                    }
                                }
                            } else if k == "user.name" {
                                // Clean up user.name if it has extra quotes
                                if let Some(name) = v.as_str() {
                                    if name.starts_with('\"') && name.ends_with('\"') {
                                        let clean_name = &name[1..name.len()-1];
                                        fields.insert(k.clone(), json!(clean_name));
                                        continue;
                                    }
                                }
                            }
                            
                            // Add other fields as-is
                            fields.insert(k.clone(), v.clone());
                        }
                    }
                    
                    // Create our custom event object
                    let event = CustomJsonEvent {
                        timestamp,
                        level,
                        target,
                        message,
                        fields,
                    };
                    
                    // Serialize to clean JSON
                    if let Ok(json) = serde_json::to_string(&event) {
                        return io::stdout().write_all((json + "\n").as_bytes()).map(|_| input_len);
                    }
                }
            }
            
            // Fall back to original JSON if parsing fails
            if !json_str.trim().is_empty() {
                return io::stdout().write_all(json_str.as_bytes()).map(|_| input_len);
            }
        }
        
        // Last resort fallback
        io::stdout().write_all(buf).map(|_| input_len)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
}

// Using the new span attribute to create a tracing span that passes context to all functions
#[params(span, fields(user.id, user.name))]
fn process_user(user: User) {
    info!("Started processing user");

    // Function calls will inherit the span context
    validate_user(&user);
    
    // Demonstrate deeper nested context propagation
    process_complex_task(user.clone());

    info!("Finished processing user");
}

fn validate_user(_user: &User) {
    // The span context from process_user will be visible here
    debug!("Validating user data");
    info!("User validation successful");
}

// Add another parameter attribute with span to demonstrate nested spans
#[params(span)]
fn process_complex_task(user: User) {
    // We'll create a span manually with additional fields
    let task_type = "user_processing";
    let operation = "verification";
    
    // The parent span context is still available, and we add our own fields inline
    warn!(task_type = task_type, operation = operation, "Starting complex task processing");
    
    // Call update_user within this context
    update_user(user.clone(), "data_sync");
    
    // Demonstrate async task with span context
    let background_task = async {
        debug!("Background task started");
        // Do some async work
        error!("Encountered an issue in background task");
        debug!("Background task completed");
    };
    
    // The async task inherits the span context
    // In a real app with tokio, you would use tokio::spawn(background_task)
    let _result = background_task.instrument(tracing::info_span!("background_job", task_type = task_type, operation = operation))
        .now_or_never();
    
    info!(task_type = task_type, operation = operation, "Complex task processing completed");
}

fn update_user(_user: User, operation_type: &str) {
    // Both parent spans' context will be visible here
    info!(operation_type = operation_type, "Updating user records");
    debug!("User records updated");
}

fn main() {
    // Set up a custom JSON formatter pipe
    let output_pipe = JsonProcessor {};
    
    // Configure the standard tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || output_pipe.clone())
        .json()
        .with_span_events(FmtSpan::NONE) // Don't include span enter/exit events
        .init();

    let user = User {
        id: 123,
        name: String::from("Alice"),
    };

    // This will create a span and log all user fields
    process_user(user);
    
    // To demonstrate that the span is properly closed/completed
    info!("All processing complete");
}
