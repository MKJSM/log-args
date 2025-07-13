# log-args

[![Crates.io](https://img.shields.io/crates/v/log-args.svg)](https://crates.io/crates/log-args)
[![Docs.rs](https://docs.rs/log-args/badge.svg)](https://docs.rs/log-args)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MKJSM/log-args/blob/main/LICENSE)
[![Build Status](https://github.com/MKJSM/log-args/actions/workflows/publish.yml/badge.svg)](https://github.com/MKJSM/log-args/actions)

A procedural macro to automatically log function arguments using the [`tracing`](https://crates.io/crates/tracing) crate.

## Overview

This crate provides the `#[params]` attribute macro, which can be applied to functions to automatically log their arguments. It enriches your logs with contextual data, making debugging and monitoring more effective.

## Features

- **Automatic Argument Logging**: Log all function arguments by default.
- **Selective Logging**: Choose specific arguments or fields to log using `fields(user.id)`.
- **Context Propagation**: Use `#[params(span)]` to propagate arguments to nested function calls, creating a logical logging scope.
- **Seamless Integration**: Works with the entire `tracing` ecosystem.
- **Async Support**: Fully compatible with `async` functions.

## Example Usage

### Basic Logging

```rust
use log_args::params;
use tracing::{debug, info};

#[params]
fn process_data(data: &str, count: u32) {
    info!("Processing data");
}
// Log output will include: `data="example" count=42`
```

### Context Propagation with `span`

Use `#[params(span)]` to create a logging context that is passed to sub-functions.

```rust
use log_args::params;
use tracing::{debug, info};

#[params(span)]
fn outer_task(task_id: i32) {
    info!("Starting outer task");
    inner_task("sub-task-1");
}

#[params]
fn inner_task(name: &str) {
    debug!("Executing inner task");
}

// When `outer_task(123)` is called, the log for `inner_task` will be:
// DEBUG ...: Executing inner task task_id=123 name="sub-task-1"
```

## Attribute Options

- `fields(arg1, arg2, ...)`: Logs only the specified arguments. If not provided, all arguments are logged.
- `span`: When present (`#[params(span)]`), arguments from this function are propagated to any `#[params]`-annotated functions it calls. This context is automatically cleared when the function goes out of scope.

## Runtime

This macro requires the companion crate [`log-args-runtime`](https://crates.io/crates/log-args-runtime), which is included as a dependency.

## License

Licensed under MIT or Apache-2.0.

**Best practices:**
- Use `span = true` for top-level or important functions to group related logs.
- For deeply nested or performance-critical code, use selectively to avoid excessive span creation.

---

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

## ❗ Limitations

* Logging context is **local to the annotated function**.
* Subfunctions **do not inherit** logged fields. To share context across calls, use `tracing::span!` manually.
* Field expressions like `user.name.first` (deep chaining) are not yet supported.

---

## 🧪 Testing & Test Coverage

This macro is tested using [`trybuild`](https://docs.rs/trybuild), covering the following:

| Test Case                        | Description                                                        |
| -------------------------------- | ------------------------------------------------------------------ |
| ✅ All arguments                  | Logs all function inputs                                           |
| ✅ Selected fields                | Logs only selected parameters                                      |
| ✅ Subfield logging               | Logs nested fields like `user.id`                                  |
| ✅ Custom fields                  | Includes hardcoded `"key" = "value"` pairs                       |
| ✅ Async function support         | Works with `async fn`                                              |
| ✅ Invalid input compile errors   | Ensures robust syntax validation                                   |
| ✅ Automatic log propagation      | Logs in subfunctions include parent context (with `#[params]`)     |

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
