# Log Args Macro - Comprehensive Usage Guide

This comprehensive guide covers all features and usage patterns of the `log_args` procedural macro library for automatic function argument logging with **truly automatic context inheritance** across all boundaries.

## Table of Contents

1. [🌟 Automatic Context Inheritance](#-automatic-context-inheritance)
2. [Basic Usage](#basic-usage)
3. [Selective Field Logging](#selective-field-logging)
4. [Custom Fields](#custom-fields)
5. [Span Context Propagation](#span-context-propagation)
6. [Function Name Logging](#function-name-logging)
7. [Async Functions](#async-functions)
8. [Error Handling](#error-handling)
9. [Method Support](#method-support)
10. [Advanced Patterns](#advanced-patterns)
11. [Security Best Practices](#security-best-practices)
12. [Performance Considerations](#performance-considerations)
13. [Troubleshooting](#troubleshooting)

## 🌟 Automatic Context Inheritance

**The killer feature of log_args**: Child functions automatically inherit parent context with just `#[params]` - no manual code changes needed!

### How It Works

When a parent function uses `#[params(span, custom(...))]`, the custom field values are automatically stored globally. Any child function with just `#[params]` will inherit ALL parent context automatically, even across complex boundaries like:

- ✅ Closure boundaries
- ✅ Async spawn boundaries  
- ✅ WebSocket upgrade closures
- ✅ Tokio task spawns
- ✅ Move closures
- ✅ Any other async/closure boundary

### Key Benefits

✅ **Zero Code Changes**: Child functions need only `#[params]` - no manual context handling  
✅ **Cross-Boundary**: Works across closures, async spawns, WebSocket upgrades, and more  
✅ **Automatic**: Context propagation happens transparently in the library  
✅ **Robust**: No more `context="<no_context>"` in your logs  
✅ **Production-Ready**: Handles complex async scenarios seamlessly  
✅ **Unlimited Nesting**: Works with deeply nested function calls  
✅ **Multiple Context Fields**: Supports multiple custom fields simultaneously  

### Multiple Context Fields

```rust
// Parent sets multiple context fields
#[params(span, custom(
    company_id = auth_user.company_id.clone(),
    user_id = auth_user.user_id.clone(),
    session_id = session.id.clone()
))]
pub async fn complex_handler(auth_user: AuthUser, session: Session) {
    info!("Complex operation started");
    
    // All child functions inherit ALL context fields
    child_operation().await;
}

#[params]  // ← Inherits company_id, user_id, AND session_id
async fn child_operation() {
    info!("Child operation");
    // Output will include all three context fields!
}
```

**Output:**
```json
{"message":"Complex operation started","company_id":"123","user_id":"456","session_id":"789"}
{"message":"Child operation","context":"company_id=123,user_id=456,session_id=789"}
```

### Migration from Manual Context Handling

**Before (Manual):**
```rust
// Old way - lots of manual work
#[params(span, custom(company_id = auth_user.company_id))]
pub async fn ws_handler(...) {
    ws.on_upgrade(with_context_capture1(move |socket| {  // ← Manual helper needed
        let client = SyncWSClient::new_with_context(     // ← Manual context passing
            db, company_id, client_id, 
            get_current_context()                        // ← Manual context retrieval
        );
        client.run(socket, broker)
    }))
}

#[params(custom(company_id = context.get("company_id")))]  // ← Manual context extraction
fn new_with_context(db: Arc<DBManager>, company_id: String, client_id: String, context: Context) -> Self {
    // Manual context handling everywhere
}
```

**After (Automatic):**
```rust
// New way - completely automatic!
#[params(span, custom(company_id = auth_user.company_id))]
pub async fn ws_handler(...) {
    ws.on_upgrade(move |socket| {                        // ← No helper needed
        let client = SyncWSClient::new(db, company_id, client_id);  // ← Normal constructor
        client.run(socket, broker)                       // ← Just works!
    })
}

#[params]  // ← Just this! Context inherited automatically
fn new(db: Arc<DBManager>, company_id: String, client_id: String) -> Self {
    // Zero manual context handling needed
}
```

## Basic Usage

The simplest way to use `log_args` is with the `#[params]` attribute:

```rust
use log_args::params;
use tracing::info;

#[params]
fn authenticate_user(username: String, password: String) {
    info!("User authentication attempt");
    // Your function logic here
}
```

**Output:**
```json
{
  "timestamp": "2023-12-07T10:30:00.123Z",
  "level": "INFO",
  "fields": {
    "message": "User authentication attempt",
    "username": "alice",
    "password": "secret123"
  },
  "target": "my_app"
}
```

This automatically logs all function parameters when the function is called.

## Selective Field Logging

For security and performance reasons, you should log only specific fields:

```rust
#[params(fields(user.id, user.name, operation_type))]
fn update_user_profile(
    user: User,
    new_email: String,     // Not logged - could be sensitive
    password: String,      // Not logged - sensitive
    operation_type: String,
    api_key: ApiKey,       // Not logged - contains secrets
) {
    info!("Updating user profile");
}
```

**Output:**
```json
{
  "fields": {
    "message": "Updating user profile",
    "user.id": "12345",
    "user.name": "Alice Johnson",
    "operation_type": "profile_update"
  }
}
```

### Complex Field Expressions

You can use complex expressions in field specifications:

```rust
#[params(fields(
    user.contact.email,
    data.len(),
    config.settings.get("timeout").unwrap_or(&30),
    metadata.get("priority").map(|p| p.as_str()).unwrap_or("normal")
))]
fn process_data(user: User, data: Vec<u8>, config: Config, metadata: HashMap<String, Value>) {
    info!("Processing data");
}
```

### Nested Field Access

```rust
#[params(fields(
    request.headers.get("user-agent"),
    payload.user.profile.preferences.theme,
    context.session.expires_at.timestamp()
))]
fn handle_request(request: HttpRequest, payload: RequestPayload, context: RequestContext) {
    info!("Handling HTTP request");
}
```

## Custom Fields

Add static metadata to your logs for service identification and debugging:

```rust
#[params(custom(
    service = "user-management",
    version = "2.1.0",
    environment = "production",
    team = "backend",
    component = "authentication"
))]
fn authenticate_user(username: String, password_hash: String) {
    info!("User authentication attempt");
}
```

**Output:**
```json
{
  "fields": {
    "message": "User authentication attempt",
    "username": "alice",
    "password_hash": "$2b$12$...",
    "service": "user-management",
    "version": "2.1.0",
    "environment": "production",
    "team": "backend",
    "component": "authentication"
  }
    span
)]
fn process_request(user_id: u64, data: String, debug_trace: String, request_id: String) {
    info!("Processing request");
    // Logs: {"user_id": 123, "debug_trace": "trace_123", "request_id": "req_456"}
    
    child_function(); // Child only inherits: {"user_id": 123} (NOT debug_trace, request_id)
}

fn child_function() {
    info!("Child function called");
    // Logs: {"user_id": 123} (inherited from parent span)
}
```

## Span Context Propagation

### Basic Span Usage

```rust
#[params(span)]
fn parent_function(user_id: u64, session_id: String) {
    info!("Parent function started");
    // Creates span with all parameters
    
    child_function();
    nested_child_function();
}

fn child_function() {
    info!("Child function called");
    // Automatically inherits: {"user_id": 123, "session_id": "sess_456"}
}

fn nested_child_function() {
    info!("Nested child function called");
    // Also inherits: {"user_id": 123, "session_id": "sess_456"}
}
```

### Selective Span Propagation

```rust
#[params(span, fields(user_id), custom(service = "payment"))]
fn process_payment(user_id: u64, amount: f64, credit_card: String, cvv: String) {
    info!("Processing payment");
    // Logs: {"user_id": 123, "service": "payment"}
    
    validate_payment();
    charge_card();
}

fn validate_payment() {
    info!("Validating payment details");
    // Inherits: {"user_id": 123, "service": "payment"}
    // Does NOT inherit: amount, credit_card, cvv (not in fields/custom)
}
```

## Async Function Support

### Basic Async Support

```rust
#[params]
async fn fetch_user_orders(user_id: u64, limit: u32) {
    info!("Starting to fetch user orders");
    
    let orders = database::fetch_orders(user_id, limit).await;
    
    info!("Orders fetched successfully");
    // Both log entries include: {"user_id": 123, "limit": 50}
}
```

### Clone Upfront for Async Safety

```rust
#[params(clone_upfront)]
async fn process_large_dataset(data: Vec<String>, config: ProcessingConfig) {
    info!("Starting data processing");
    // Parameters are cloned upfront to avoid ownership issues
    
    for chunk in data.chunks(100) {
        process_chunk(chunk).await;
        info!("Chunk processed");
        // All log entries include the original parameters
    }
    
    info!("Data processing completed");
}
```

### Async with Selective Fields

```rust
#[params(fields(user_id, operation), clone_upfront)]
async fn async_user_operation(
    user_id: u64, 
    operation: String, 
    sensitive_data: String, 
    api_credentials: ApiCredentials
) {
    info!("Starting async operation");
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    info!("Async operation completed");
    // Both logs include: {"user_id": 123, "operation": "data_export"}
    // Excludes: sensitive_data, api_credentials
}
```

## Advanced Field Expressions

### Method Calls and Complex Expressions

```rust
#[params(fields(
    user.name.len(),
    user.emails.is_empty(),
    config.settings.keys().count(),
    data.created_at.timestamp()
))]
fn analyze_user_data(user: User, config: Config, data: UserData, password: String) {
    info!("Analyzing user data");
    // Logs: {
    //   "user.name.len()": 12,
    //   "user.emails.is_empty()": false,
    //   "config.settings.keys().count()": 5,
    //   "data.created_at.timestamp()": 1640995200
    // }
    // Excludes: password
}
```

### Custom Fields with Expressions

```rust
#[params(
    fields(user.id),
    custom(
        account_age_days = (now() - user.created_at).num_days(),
        is_premium = user.subscription.tier == "premium",
        total_orders = user.orders.len()
    )
)]
fn process_user_account(user: User, now: DateTime, api_key: String) {
    info!("Processing user account");
    // Logs: {
    //   "user.id": 123,
    //   "account_age_days": 365,
    //   "is_premium": true,
    //   "total_orders": 42
    // }
    // Excludes: api_key
}
```

## Combining Attributes

### All Attributes Combined

```rust
#[params(
    fields(user.id, user.email),
    custom(service = "notification", version = "3.0"),
    current(trace_id, debug_info),
    span,
    clone_upfront
)]
async fn send_notification(
    user: User,
    message: String,
    api_key: String,
    trace_id: String,
    debug_info: String
) {
    info!("Sending notification");
    // Current function logs: {
    //   "user.id": 123,
    //   "user.email": "alice@example.com",
    //   "service": "notification",
    //   "version": "3.0",
    //   "trace_id": "trace_123",
    //   "debug_info": "debug_abc"
    // }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    deliver_message().await;
    
    info!("Notification sent successfully");
}

async fn deliver_message() {
    info!("Delivering message");
    // Child function inherits: {
    //   "user.id": 123,
    //   "user.email": "alice@example.com", 
    //   "service": "notification",
    //   "version": "3.0"
    // }
    // Does NOT inherit: trace_id, debug_info (marked as current)
}
```

## Best Practices

### 1. Security-First Field Selection

```rust
// ✅ Good: Only log necessary fields
#[params(fields(user_id, action, timestamp))]
fn audit_action(user_id: u64, action: String, timestamp: i64, password: String, api_key: String) {
    info!("Action audited");
}

// ❌ Avoid: Logging all parameters when sensitive data is present
#[params]
fn audit_action_bad(user_id: u64, action: String, timestamp: i64, password: String, api_key: String) {
    info!("Action audited"); // Logs password and api_key!
}
```

### 2. Meaningful Custom Fields

```rust
// ✅ Good: Add service context
#[params(
    fields(user_id, order_id),
    custom(service = "payment", operation = "charge", version = "2.1")
)]
fn process_payment(user_id: u64, order_id: String, amount: f64, card_token: String) {
    info!("Processing payment");
}
```

### 3. Use Current Fields for Debug Data

```rust
// ✅ Good: Debug data stays local, business data propagates
#[params(
    fields(user_id, order_id),
    current(debug_trace, performance_metrics),
    span
)]
fn process_order(user_id: u64, order_id: String, debug_trace: String, performance_metrics: String) {
    info!("Processing order");
    // Child functions get user_id, order_id but not debug data
}
```

### 4. Clone Upfront for Long-Running Async

```rust
// ✅ Good: Prevent ownership issues in async contexts
#[params(fields(user_id, job_type), clone_upfront)]
async fn long_running_job(user_id: u64, job_type: String, large_data: Vec<u8>) {
    info!("Starting long-running job");
    
    for chunk in large_data.chunks(1000) {
        process_chunk(chunk).await;
        info!("Chunk processed"); // Still has access to user_id, job_type
    }
}
```

## Security Considerations

### 1. Sensitive Data Exclusion

```rust
// Always exclude sensitive fields
#[params(fields(user_id, email))] // Exclude password, tokens, keys
fn authenticate_user(user_id: u64, email: String, password: String, session_token: String) {
    info!("Authentication attempt");
}
```

### 2. PII (Personally Identifiable Information) Handling

```rust
// Be selective with PII
#[params(fields(user_id))] // Include ID, exclude name/email for privacy
fn process_analytics(user_id: u64, full_name: String, email: String, phone: String) {
    info!("Processing analytics");
}
```

### 3. API Keys and Credentials

```rust
// Never log credentials
#[params(fields(endpoint, timeout_ms))]
fn external_api_call(endpoint: String, api_key: String, secret: String, timeout_ms: u64) {
    info!("Making external API call");
}
```

### 4. Financial and Health Data

```rust
// Exclude sensitive financial/health data
#[params(fields(user_id, transaction_id))]
fn process_payment(
    user_id: u64, 
    transaction_id: String, 
    credit_card_number: String, 
    cvv: String,
    ssn: String
) {
    info!("Processing payment");
}
```

This comprehensive guide covers all aspects of using the `log-args` macro safely and effectively in both synchronous and asynchronous Rust applications.
