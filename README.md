
# log_args

[![Crates.io](https://img.shields.io/crates/v/log-args.svg)](https://crates.io/crates/log-args)
[![Docs.rs](https://docs.rs/log-args/badge.svg)](https://docs.rs/log-args)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MKJSM/log-args/blob/main/LICENSE)
[![Build Status](https://github.com/MKJSM/log-args/actions/workflows/publish.yml/badge.svg)](https://github.com/MKJSM/log-args/actions)

A procedural macro for structured logging of function arguments using the `tracing` crate, with special support for async contexts.

This crate provides a procedural macro attribute `#[params]` that can be applied to functions to automatically log their arguments. It seamlessly integrates with the `tracing` ecosystem for structured logging, offering enhanced flexibility for both synchronous and asynchronous functions.

---

## 🚀 Overview

- **Macro name:** `#[params]`
- **Purpose:** Automatically logs function arguments and custom fields using the `tracing` macros (`info!`, `debug!`, etc.)
- **Works with:** Both synchronous and asynchronous functions, with special support for async contexts
- **Flexible logging:** Control exactly what fields are logged, including nested struct fields
- **Async-friendly:** Special `clone_upfront` option for handling ownership in async blocks

---

## ✨ Features

- **Log all function arguments** by default
- **Select specific arguments** to log using `fields(...)`
- **Log nested fields** of struct arguments (e.g., `user.id`)
- **Add custom key-value pairs** to the log output using `custom(...)`
- **Async support** with special `clone_upfront` option for ownership in async contexts
- Supports both **synchronous and asynchronous functions**
- All logging is done through the `tracing` macros (`info!`, `debug!`, `warn!`, `error!`, `trace!`)
- **No `level` attribute**: Use the desired `tracing` macro directly in your function body

---

## 📦 Installation

Add `log_args` to your `Cargo.toml`:

```toml
[dependencies]
log_args = "0.1" # Replace with the latest version from crates.io
tracing = "0.1"
```

You'll also need a compatible `tracing` subscriber to process and display logs. For example:

```rust
// Initialize a simple console subscriber
tracing_subscriber::fmt().init();
```

---

## 🔧 Basic Usage

### Log All Arguments

By default, `#[params]` logs all arguments of a function.

```rust
use log_args::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO process_user: Processing task user=User { id: 42, name: "Alice" } task_id=100
```

### Log Specific Fields

You can specify which arguments or fields to log using the `fields(...)` attribute:

```rust
use log_args::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params(fields(user.id, user.name))]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO process_user: Processing task user.id=42 user.name="Alice"
```

### Add Custom Key-Value Pairs

Use `custom(...)` to add static key-value pairs to your logs:

```rust
use log_args::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params(custom(service_name = "user-service", version = "1.0"))]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO process_user: Processing task user=User { id: 42, name: "Alice" } task_id=100 service_name="user-service" version="1.0"
```

### Combine Multiple Options

You can combine `fields` and `custom` options:

```rust
use log_args::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params(fields(user.id), custom(service_name = "user-service"))]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO process_user: Processing task user.id=42 service_name="user-service"
```

---

## 🔄 Advanced Usage: Async Functions

The `#[params]` macro works seamlessly with async functions:

```rust
use log_args::params;
use tracing::info;
use tokio::time::sleep;
use std::time::Duration;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params]
async fn process_user_async(user: User, task_id: i32) {
    info!("Starting async task");
    sleep(Duration::from_millis(100)).await;
    info!("Completed async task");
}
```

### Using `clone_upfront` for Async Contexts

When working with async code, especially when moving values into `async move` blocks or `tokio::spawn`, you might encounter ownership issues. The `clone_upfront` option addresses this by ensuring fields can be safely used throughout your async function:

```rust
use log_args::params;
use tracing::info;

#[derive(Debug, Clone)]
struct Client { id: String, name: String }

#[params(clone_upfront, fields(client.id, client.name))]
async fn process_client(client: Client) {
    info!("Starting client processing");
    
    // Move client into an async block
    let task = tokio::spawn(async move {
        // Use client here without ownership issues
        info!("Processing client in spawned task");
        client
    });
    
    // Logs still work even though client was moved
    // because values were cloned upfront
    info!("Waiting for client processing to complete");
}
```

The `clone_upfront` option is particularly useful when:
- You need to move values into `async move` blocks
- You're using `tokio::spawn` or similar functions
- You want to log values even after they've been moved

---

## 🛠️ Feature Reference

### Attribute Options

| Option | Description | Example |
|--------|-------------|----------|
| `fields(...)` | Specify which fields to log | `#[params(fields(user.id, count))]` |
| `custom(...)` | Add custom key-value pairs | `#[params(custom(version = "1.0"))]` |
| `clone_upfront` | Clone fields for safe use in async contexts | `#[params(clone_upfront)]` |

### Logging Options

The `#[params]` macro redefines the following `tracing` macros within the function body:
- `info!`
- `warn!`
- `error!`
- `debug!`
- `trace!`

Use these macros as you normally would - the function arguments will be automatically included in the output.

---

## 📋 Examples

Check out the examples directory for more detailed usage patterns:

- `examples/basic.rs`: Basic usage with all arguments
- `examples/selected_fields.rs`: Logging specific fields
- `examples/custom_fields.rs`: Adding custom key-value pairs
- `examples/async_function.rs`: Usage with async functions
- `examples/async_clone_upfront.rs`: Using `clone_upfront` with async functions
- `examples/subfields.rs`: Logging nested struct fields

---

## 🔍 How It Works

The `#[params]` macro:

1. Analyzes the function signature to find available arguments
2. Processes attribute options like `fields(...)` and `custom(...)`
3. Redefines tracing macros within the function scope to automatically include the specified fields
4. With `clone_upfront`, ensures values are safely cloned to prevent ownership issues in async contexts

The macro does not add overhead beyond the normal cost of logging and cloning when needed.

---

## ⚠️ Limitations

- The `#[params]` macro redefines tracing macros within function scope, which may generate unused macro warnings if not all redefined macros are used (these are suppressed internally)
- When using `clone_upfront`, fields must implement `Clone`
- Deeply nested or complex field expressions may not be properly captured
- Currently, directly using a struct in a field expression (e.g., `fields(user)` instead of `fields(user.id)`) may not work as expected; use individual fields instead

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🧵 Example: End-to-End

```rust
use log_args::log_args;
use tracing_subscriber;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

#[log_args(fields(user.id, user.name), custom("service" = "auth"))]
fn login(user: User) {
    info!("Login started");
    warn!("Invalid password");
}
```

### Logs:

```
INFO login: user_id=42 user_name="Alice" service="auth" Login started
WARN login: user_id=42 user_name="Alice" service="auth" Invalid password
```

---

**❌ Incorrect usage:**
```rust
#[params]
fn foo() {
    tracing::debug!("debug message"); // will NOT be enriched
}
```

**Always import the macros you use:**
```rust
use tracing::{debug, info, warn, error};
```

---

## 🔮 Future Enhancements

* `#[log_args(span = true)]`: Optional span-based logging for subfunction support
* `#[log_args(log_return)]`: Auto-log return values
* Integration with `opentelemetry` and structured span hierarchy

---

## ✅ License

MIT or Apache 2.0 — your choice.

---

## 🙌 Contributions

PRs, issues, and feedback are welcome. Let’s make logging in Rust ergonomic and powerful, together.

---

## 📫 Contact

Maintained by \[MKJS Tech](mailto:mkjsm57@gmail.com) • Feel free to reach out via [mail](mailto:mkjsm57@gmail.com) or [GitHub Issues](https://github.com/MKJSM/log-args/issues).
