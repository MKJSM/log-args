# params

[![Crates.io](https://img.shields.io/crates/v/params.svg)](https://crates.io/crates/log-args)
[![Docs.rs](https://docs.rs/log-args/badge.svg)](https://docs.rs/log-args)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MKJSM/log-args/blob/main/LICENSE)
[![Build Status](https://github.com/MKJSM/log-args/actions/workflows/publish.yml/badge.svg)](https://github.com/MKJSM/log-args/actions)

A simple procedural macro to log function arguments using the `tracing` crate.

This crate provides a procedural macro attribute `#[params]` that can be applied to functions to automatically log their arguments. It is designed to be simple, efficient, and easy to integrate into any project that uses `tracing` for structured logging.

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

Add `params` to your `Cargo.toml`:

```toml
[dependencies]
params = "0.1.0" # Replace with the latest version from crates.io
tracing = "0.1"
tracing-attributes = "0.1"
```

---

## 🔧 Usage

### 1. Log All Arguments

By default, `#[params]` logs all arguments of a function.

```rust
use params::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32 }

#[params]
fn process_user(user: User, task_id: i32) {
    info!("Processing task");
}

// Log output will be similar to:
// INFO Processing task user=User { id: 42 } task_id=100
```

### 2. Log Specific Fields

Use `fields(...)` to select which arguments or subfields to log.

```rust
use params::params;
use tracing::warn;

#[derive(Debug)]
struct User { id: u32, name: String }

#[params(fields(user.id))]
fn process_user(user: User) {
    warn!("Processing failed");
}

// Log output will be similar to:
// WARN Processing failed user_id=42
```

### 3. Add Custom Key-Value Pairs

Use `custom(...)` to add static key-value pairs to your logs.

```rust
use params::params;
use tracing::info;

#[derive(Debug)]
struct User { id: u32 }

#[params(fields(user.id), custom(service = "auth", env = "production"))]
fn authenticate(user: User) {
    info!("Login attempt");
}

// Log output will be similar to:
// INFO Authenticating user user_id=42 service="auth"
```


### 4. Asynchronous Functions

The macro works seamlessly with `async` functions.

```rust
use params::params;
use tracing::info;

#[derive(Debug)]
struct User { email: String }

#[params(fields(user.email))]
async fn send_email(user: User) {
    info!("Sending confirmation email");
    // ... async logic ...
}
```

### 5. Span-based Logging

You can enable span-based logging by setting `span = true` in the macro attributes. This will create a `tracing::span!` that encompasses the function's execution.

```rust
use params::params;
use tracing::info;

#[params(span = true)]
fn my_function(arg1: i32) {
    info!("Inside my_function");
}

// When called, this will produce a span for `my_function` and log its arguments within that span.
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
use params::params;
use tracing_subscriber;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

#[params(fields(user.id, user.name), custom("service" = "auth"))]
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

* `#[params(log_return)]`: Auto-log return values

---

## ✅ License

MIT or Apache 2.0 — your choice.

---

## 🙌 Contributions

PRs, issues, and feedback are welcome. Let’s make logging in Rust ergonomic and powerful, together.

---

## 📫 Contact

Maintained by \[MKJS Tech](mailto:mkjsm57@gmail.com) • Feel free to reach out via [mail](mailto:mkjsm57@gmail.com) or [GitHub Issues](https://github.com/MKJSM/log-args/issues).
