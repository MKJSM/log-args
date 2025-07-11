
# log_args

[![Crates.io](https://img.shields.io/crates/v/log-args.svg)](https://crates.io/crates/log-args)
[![Docs.rs](https://docs.rs/log-args/badge.svg)](https://docs.rs/log-args)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MKJSM/log-args/blob/main/LICENSE)
[![Build Status](https://github.com/MKJSM/log-args/actions/workflows/publish.yml/badge.svg)](https://github.com/MKJSM/log-args/actions)

A simple procedural macro to log function arguments using the `tracing` crate.

This crate provides a procedural macro attribute `#[log_args]` that can be applied to functions to automatically log their arguments. It is designed to be simple, efficient, and easy to integrate into any project that uses `tracing` for structured logging.

---

## ✨ Features

- **Log all function arguments** by default.
- **Select specific arguments** to log using `fields(...)`.
- **Log nested fields** of struct arguments (e.g., `user.id`).
- **Add custom key-value pairs** to the log output using `custom(...)`.
- Supports both **synchronous and asynchronous functions**.
- All logging is done through the `tracing` ecosystem, which means it has **zero-overhead** when disabled.

---

## 📦 Installation

Add `log_args` to your `Cargo.toml`:

```toml
[dependencies]
log_args = "0.1.0" # Replace with the latest version from crates.io
tracing = "0.1"
```

---

## 🔧 Usage

### 1. Log All Arguments

By default, `#[log_args]` logs all arguments of a function.

```rust
use log_args::log_args;
use tracing::info;

#[derive(Debug)]
struct User { id: u32 }

#[log_args]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO Processing task user=User { id: 42 } task_id=100
```

### 2. Log Specific Fields

Use `fields(...)` to select which arguments or subfields to log.

```rust
use log_args::log_args;
use tracing::warn;

#[derive(Debug)]
struct User { id: u32, name: String }

#[log_args(fields(user.id))]
fn process_user(user: User) {
    warn!("Processing failed");
}

// Log output will be similar to:
// WARN Processing failed user_id=42
```

### 3. Add Custom Key-Value Pairs

Use `custom(...)` to add static key-value pairs to your logs.

```rust
use log_args::log_args;
use tracing::info;

#[derive(Debug)]
struct User { id: u32 }

#[log_args(fields(user.id), custom(service = "auth", env = "production"))]
fn authenticate(user: User) {
    info!("Login attempt");
}

// Log output will be similar to:
// INFO Login attempt user_id=42 service="auth" env="production"
```

### 4. Asynchronous Functions

The macro works seamlessly with `async` functions.

```rust
use log_args::log_args;
use tracing::info;

#[derive(Debug)]
struct User { email: String }

#[log_args(fields(user.email))]
async fn send_email(user: User) {
    info!("Sending confirmation email");
    // ... async logic ...
}
```

For more detailed examples, please see the [examples directory](https://github.com/MKJSM/log-args/tree/main/examples) in the repository.

---

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/MKJSM/log-args/blob/main/LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.


---

## ⚙️ How It Works

* The macro injects helper logging macros (`info!`, `warn!`, etc.) directly into the function scope.
* These macros automatically attach the specified `fields(...)` and `custom(...)` values to every log.

Example:

```rust
info!("Welcome");

↓ expands to ↓

tracing::info!(
    user_id = ?user.id,
    service = "auth",
    "Welcome"
);
```

This makes logging consistent and easy to use without spans or boilerplate.

---

## ❗ Limitations

* Logging context is **local to the annotated function**.
* Subfunctions **do not inherit** logged fields. To share context across calls, use `tracing::span!` manually.
* Field expressions like `user.name.first` (deep chaining) are not yet supported.

---

## 🧪 Testing & Test Coverage

This macro is tested using [`trybuild`](https://docs.rs/trybuild), covering the following:

| Test Case                      | Description                                |
| ------------------------------ | ------------------------------------------ |
| ✅ All arguments                | Logs all function inputs                   |
| ✅ Selected fields              | Logs only selected parameters              |
| ✅ Subfield logging             | Logs nested fields like `user.id`          |
| ✅ Custom fields                | Includes hardcoded `"key" = "value"` pairs |
| ✅ Async function support       | Works with `async fn`                      |
| ✅ Invalid input compile errors | Ensures robust syntax validation           |
| ❌ No automatic log propagation | Logs in subfunctions won't include context |

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

Maintained by \[YourNameHere] • Feel free to reach out via GitHub Issues.
