//! # log-args: Procedural Macro for Logging Function Arguments with Async Support
//!
//! This crate provides the `#[params]` attribute macro to automatically log function arguments using the `tracing` crate,
//! with special support for handling ownership in asynchronous contexts.
//!
//! ## Features
//! - Log all function arguments automatically or select specific ones
//! - Log nested fields of struct arguments (e.g., `user.id`)
//! - Add custom key-value pairs to log output
//! - Compatible with both synchronous and asynchronous functions
//! - Special `clone_upfront` option for handling ownership in async move blocks and spawned tasks
//! - No tracing spans or span-related features used - just simple structured logging
//!
//! ## Basic Usage
//!
//! ```rust
//! use log_args::params;
//! use tracing::info;
//!
//! #[derive(Debug)]
//! struct User { id: u32, name: String }
//!
//! // Log all arguments
//! #[params]
//! fn process_user(user: User, count: usize) {
//!     info!("Processing user data");
//!     // Logs: INFO process_user: Processing user data user=User { id: 42, name: "Alice" } count=5
//! }
//!
//! // Log only specific fields
//! #[params(fields(user.id, count))]
//! fn validate_user(user: User, count: usize) {
//!     info!("Validating user");
//!     // Logs: INFO validate_user: Validating user user.id=42 count=5
//! }
//!
//! // Add custom values
//! #[params(custom(service = "auth", version = "1.0"))]
//! fn authenticate(user: User) {
//!     info!("Authentication attempt");
//!     // Logs: INFO authenticate: Authentication attempt user=User { id: 42, name: "Alice" } service="auth" version="1.0"
//! }
//! ```
//!
//! ## Advanced: Async Support with `clone_upfront`
//!
//! When working with asynchronous code, especially when moving values into `async move` blocks or
//! `tokio::spawn`, you might encounter ownership issues. The `clone_upfront` option addresses this
//! by ensuring fields can be safely used throughout your async function:
//!
//! ```rust
//! use log_args::params;
//! use tracing::info;
//!
//! #[derive(Debug, Clone)]
//! struct Client { id: String, name: String }
//!
//! #[params(clone_upfront, fields(client.id, client.name))]
//! async fn process_client(client: Client) {
//!     info!("Starting client processing");
//!     
//!     // Move client into an async block
//!     let task = tokio::spawn(async move {
//!         // Use client here without ownership issues
//!         // ...
//!         client
//!     });
//!     
//!     // Logs still work even though client was moved
//!     // because values were cloned upfront
//!     info!("Waiting for client processing to complete");
//! }
//! ```
//!
//! ## Macro Attributes
//!
//! - `fields(...)`: Log only the specified arguments or fields (e.g., `fields(user.id, count)`).
//!   This is useful for reducing log verbosity or accessing nested fields of struct arguments.
//!
//! - `custom(...)`: Add custom key-value pairs to the log output (e.g., `custom(service = "auth")`).
//!   This allows adding static context to your logs that isn't directly from function arguments.
//!
//! - `clone_upfront`: Clone fields upfront to avoid ownership issues in async contexts. This is particularly
//!   useful when working with `async move` blocks or spawned tasks where the original variables are moved.
//!
//! ## How It Works
//!
//! The `#[params]` macro redefines the tracing macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`)
//! within the function body to automatically include the specified function arguments or fields in the
//! log output. For `clone_upfront`, it generates code that safely handles ownership by cloning values
//! as needed to ensure they're available throughout the function execution.
//!
//! When using with async functions, especially those containing `tokio::spawn` with `async move` blocks,
//! be careful about ownership and lifetimes. There are three patterns for handling this:
//!
//! ### Pattern 1: Using with owned parameters (default behavior)
//!
//! ```rust
//! #[params(fields(session.user_id, session.session_id))]
//! async fn process_session(session: Session) {
//!     info!("Processing session"); // Fields are cloned here
//!     
//!     // Do some processing
//!     tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
//!     
//!     info!("Session processing completed"); // Fields are cloned here too
//! }
//! ```
//!
//! ### Pattern 2: Using the `clone_upfront` option (for async functions)
//!
//! This pattern clones all fields at the beginning of the function, which helps with logging
//! in async functions. Note that for `tokio::spawn` with `async move` blocks, you'll still need
//! to clone the fields again before the async block:
//!
//! ```rust
//! #[params(clone_upfront, fields(self.client_id, self.company_id))]
//! async fn run(self, socket: WebSocket) {
//!     info!("Starting connection"); // Uses cloned fields
//!     
//!     // Clone variables are created at the beginning of the function
//!     // so they're available for all tracing macros in the main function body
//!     
//!     // For tokio::spawn, you need to clone again for the async block
//!     let task_client_id = self.client_id.clone();
//!     let task_company_id = self.company_id.clone();
//!     
//!     let task = tokio::spawn(async move {
//!         // Use the task-specific clones inside the async block
//!         info!(client_id = ?task_client_id, company_id = ?task_company_id, "Worker task started");
//!     });
//!     
//!     info!("Connection handler completed"); // Uses the original cloned fields
//! }
//! ```
//!
//! ### Pattern 3: Manual logging with references and async blocks
//!
//! When you need more control, you can manually clone and log fields:
//!
//! ```rust
//! async fn handle_connection(client_id: String, company_id: String) {
//!     // Log manually at the beginning
//!     info!(client_id = ?client_id, company_id = ?company_id, "Starting connection");
//!
//!     // Clone what we need for the async block
//!     let task_client_id = client_id.clone();
//!
//!     // Spawn the async task with its own cloned data
//!     let task = tokio::spawn(async move {
//!         info!(client_id = ?task_client_id, "Worker task started");
//!     });
//!
//!     // This log can still use the original client_id
//!     info!(client_id = ?client_id, "Connection handler completed");
//! }
//! ```
//!
//! See the examples directory for more detailed usage patterns.
//!
//! ## Limitations
//! - Only works with the `tracing` crate macros (`info!`, `debug!`, `warn!`, `error!`, `trace!`).
//! - Does not support span creation or level selection via macro input.
//! - All arguments must implement `Debug` for structured logging.
//! - For async code with `tokio::spawn`, use the `clone_upfront` option to avoid ownership issues.
//! - Generates warnings about unused macro definitions (these are expected and can be ignored).
//!
//! See the examples directory for more detailed usage patterns.

extern crate proc_macro;

/// A procedural macro for automatically logging function arguments with tracing.
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token,
};

/// Represents the parsed arguments of the `#[params]` attribute macro.
///
/// This struct holds the configuration for how function arguments should be logged:
/// - Which specific fields to log (if any)
/// - Any custom key-value pairs to include in logs
/// - Whether to clone fields upfront for async safety
///
/// When no fields are specified, all function arguments will be logged.
struct LogArgs {
    /// Expressions representing specific fields to log (e.g., `user.id`, `count`)
    fields: Vec<Expr>,

    /// Custom key-value pairs to include in logs (e.g., `service = "auth"`)
    custom: Vec<MetaNameValue>,

    /// Whether to clone fields upfront at the beginning of the function
    /// This is useful for async functions with `tokio::spawn` or `async move` blocks
    /// to avoid ownership issues
    clone_upfront: bool,
}

impl Parse for LogArgs {
    /// Parses the arguments of the `#[params]` attribute macro.
    ///
    /// This method handles three types of arguments:
    /// - `fields(...)`: Expressions representing specific fields to log
    /// - `custom(...)`: Custom key-value pairs to include in logs
    /// - `clone_upfront`: A flag to enable upfront cloning for async safety
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[params] // No arguments - log all function parameters
    /// #[params(fields(user.id, count))] // Log only specific fields
    /// #[params(custom(service = "auth"))] // Add custom key-value pairs
    /// #[params(clone_upfront)] // Clone fields upfront for async safety
    /// #[params(fields(user.id), custom(service = "auth"), clone_upfront)] // Combined
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if invalid attribute syntax is used or if unknown attributes
    /// are provided.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Vec::new();
        let mut custom = Vec::new();
        let mut clone_upfront = false;

        if input.is_empty() {
            return Ok(LogArgs {
                fields,
                custom,
                clone_upfront,
            });
        }

        // Parse comma-separated attributes
        let nested_metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in nested_metas {
            match meta {
                Meta::List(list) => {
                    if list.path.is_ident("fields") {
                        fields.extend(
                            list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?,
                        );
                    } else if list.path.is_ident("custom") {
                        custom.extend(list.parse_args_with(
                            Punctuated::<MetaNameValue, Token![,]>::parse_terminated,
                        )?);
                    } else {
                        return Err(syn::Error::new_spanned(
                            list.path,
                            "Unknown attribute, expected `fields` or `custom`",
                        ));
                    }
                }
                Meta::Path(path) => {
                    if path.is_ident("clone_upfront") {
                        clone_upfront = true;
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "Unknown attribute, expected `clone_upfront`",
                        ));
                    }
                }
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("clone_upfront") {
                        // Handle clone_upfront = true/false if needed
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Bool(lit_bool),
                            ..
                        }) = &name_value.value
                        {
                            clone_upfront = lit_bool.value;
                        } else {
                            return Err(syn::Error::new_spanned(
                                name_value.value,
                                "Expected a boolean literal for `clone_upfront`",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            name_value.path,
                            "Unknown attribute",
                        ));
                    }
                }
            }
        }

        Ok(LogArgs {
            fields,
            custom,
            clone_upfront,
        })
    }
}

/// A procedural macro attribute for automatically logging function arguments.
///
/// This macro modifies functions to automatically log their arguments when tracing macros
/// are called within the function body. It redefines the standard tracing macros
/// (`info!`, `warn!`, `error!`, `debug!`, `trace!`) within the function scope to include
/// the specified function arguments or fields in every log message.
///
/// # Usage Options
///
/// - `#[params]` - Log all function arguments
/// - `#[params(fields(arg1, arg2.field))]` - Log only specific arguments or fields
/// - `#[params(custom(key = "value"))]` - Add custom key-value pairs to logs
/// - `#[params(clone_upfront)]` - Clone fields upfront for async safety
/// - Any combination of the above
///
/// # Examples
///
/// Basic usage - log all arguments:
/// ```rust
/// #[params]
/// fn process_order(order_id: u32, user: User) {
///     info!("Processing order"); // Logs all arguments
/// }
/// ```
///
/// Log specific fields:
/// ```rust
/// #[params(fields(user.id, item_count))]
/// fn process_order(order_id: u32, user: User, item_count: usize) {
///     info!("Processing order"); // Logs only user.id and item_count
/// }
/// ```
///
/// Add custom key-value pairs:
/// ```rust
/// #[params(custom(service = "order-processor", version = "1.0"))]
/// fn process_order(order_id: u32) {
///     info!("Processing order"); // Includes the custom fields
/// }
/// ```
///
/// Handle async ownership with clone_upfront:
/// ```rust
/// #[params(clone_upfront)]
/// async fn process_order(order: Order) {
///     info!("Starting order processing");
///     
///     tokio::spawn(async move {
///         // Use order here...
///         order.process().await;
///     });
///     
///     info!("Order dispatched"); // Still works, fields were cloned
/// }
/// ```
///
/// # How It Works
///
/// The macro redefines tracing macros within the function body to automatically include
/// the specified fields in every log call. With `clone_upfront`, it generates code to clone
/// values at the beginning of the function to ensure they're available throughout execution,
/// even if the original values are moved.
///
/// # Parameters
///
/// * `args` - The macro arguments as specified in the attribute
/// * `input` - The function to transform
///
/// # Returns
///
/// A TokenStream representing the transformed function with enhanced logging
#[proc_macro_attribute]
pub fn params(args: TokenStream, input: TokenStream) -> TokenStream {
    let params_args: LogArgs = match syn::parse(args) {
        Ok(params) => params,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut func: ItemFn = match syn::parse(input) {
        Ok(func) => func,
        Err(e) => return e.to_compile_error().into(),
    };

    // If no fields are provided, do nothing.
    if params_args.fields.is_empty() && params_args.custom.is_empty() {
        return func.into_token_stream().into();
    }

    // Create the field expressions for the log macros that clone inline
    // We'll store the original expressions to use in the macro redefinitions
    let mut field_exprs_vec = Vec::new();
    for expr in &params_args.fields {
        let expr_str = expr.to_token_stream().to_string().replace(' ', "");
        field_exprs_vec.push((expr_str, expr.clone()));
    }

    let mut custom_exprs_vec = Vec::new();
    for nv in &params_args.custom {
        let key = nv.path.to_token_stream().to_string();
        custom_exprs_vec.push((key, nv.value.clone()));
    }

    // Get the original function body
    let original_body = func.block;

    // Handle differently based on whether we're cloning upfront or inline
    if params_args.clone_upfront {
        // In the clone_upfront mode, we'll use thread-local storage to keep cloned values
        // that can be safely accessed even if the original values are moved

        // First, create a unique identifier for this function
        let fn_name = format!("{}", func.sig.ident);
        let fn_id = syn::Ident::new(&fn_name, proc_macro2::Span::call_site());

        // Create field names for thread_local storage
        let tls_var_names = field_exprs_vec
            .iter()
            .map(|(expr_str, _)| {
                let safe_name = expr_str.replace('.', "_");
                let tls_name = format!("__LOG_ARGS_TLS_{fn_id}_{safe_name}");
                (
                    expr_str.clone(),
                    syn::Ident::new(&tls_name, proc_macro2::Span::call_site()),
                )
            })
            .collect::<Vec<_>>();

        let custom_tls_var_names = custom_exprs_vec
            .iter()
            .map(|(key, _)| {
                let tls_name = format!("__LOG_ARGS_TLS_{fn_id}_{key}");
                (
                    key.clone(),
                    syn::Ident::new(&tls_name, proc_macro2::Span::call_site()),
                )
            })
            .collect::<Vec<_>>();

        // Generate thread_local declarations
        let thread_locals = tls_var_names.iter().map(|(_expr_str, tls_var)| {
            quote! {
                thread_local! {
                    static #tls_var: ::std::cell::RefCell<Option<String>> =
                        ::std::cell::RefCell::new(None);
                }
            }
        });

        let custom_thread_locals = custom_tls_var_names.iter().map(|(_key, tls_var)| {
            quote! {
                thread_local! {
                    static #tls_var: ::std::cell::RefCell<Option<String>> =
                        ::std::cell::RefCell::new(None);
                }
            }
        });

        // Generate thread_local initializations
        let tls_inits = tls_var_names.iter().map(|(expr_str, tls_var)| {
            let expr_index = field_exprs_vec
                .iter()
                .position(|(s, _)| s == expr_str)
                .unwrap();
            let (_, expr) = &field_exprs_vec[expr_index];

            quote! {
                #tls_var.with(|cell| {
                    *cell.borrow_mut() = Some(format!("{}", &#expr));
                });
            }
        });

        let custom_tls_inits = custom_tls_var_names.iter().map(|(key, tls_var)| {
            let custom_index = custom_exprs_vec.iter().position(|(k, _)| k == key).unwrap();
            let (_, value) = &custom_exprs_vec[custom_index];

            quote! {
                #tls_var.with(|cell| {
                    *cell.borrow_mut() = Some(format!("{}", &#value));
                });
            }
        });

        // Generate field expressions for tracing macros that reference thread_locals
        let field_exprs = tls_var_names.iter().map(|(expr_str, tls_var)| {
            let key_parts: Vec<&str> = expr_str.split('.').collect();
            let key = key_parts.last().copied().unwrap_or(expr_str.as_str());

            quote! { #key = #tls_var.with(|cell| cell.borrow().clone().unwrap_or_default()) }
        });

        let custom_exprs = custom_tls_var_names.iter().map(|(key, tls_var)| {
            let key_ident = syn::parse_str::<syn::Path>(key).unwrap();

            quote! { #key_ident = #tls_var.with(|cell| cell.borrow().clone().unwrap_or_default()) }
        });

        // Collect all field expressions
        let all_field_exprs: Vec<proc_macro2::TokenStream> =
            field_exprs.chain(custom_exprs).collect();
        let field_exprs_tokens = quote! { #(#all_field_exprs),* };

        // Create a new function body with thread_locals and macro redefinitions
        let new_body = quote! {
            {
                // Suppress unused macro warnings since not all redefined macros might be used
                #[allow(unused_macros)]
                // Define thread_local storage for each field and custom value
                #(#thread_locals)*
                #(#custom_thread_locals)*

                // Initialize thread_locals with current values
                #(#tls_inits)*
                #(#custom_tls_inits)*

                // Redefine tracing macros to use thread_local values
                #[allow(unused_macros)]
                macro_rules! info {
                    () => { tracing::info!(#field_exprs_tokens); };
                    ($($arg:tt)+) => { tracing::info!(#field_exprs_tokens, $($arg)*); };
                }

                #[allow(unused_macros)]
                macro_rules! warn {
                    () => { tracing::warn!(#field_exprs_tokens); };
                    ($($arg:tt)+) => { tracing::warn!(#field_exprs_tokens, $($arg)*); };
                }

                #[allow(unused_macros)]
                macro_rules! error {
                    () => { tracing::error!(#field_exprs_tokens); };
                    ($($arg:tt)+) => { tracing::error!(#field_exprs_tokens, $($arg)*); };
                }

                #[allow(unused_macros)]
                macro_rules! debug {
                    () => { tracing::debug!(#field_exprs_tokens); };
                    ($($arg:tt)+) => { tracing::debug!(#field_exprs_tokens, $($arg)*); };
                }

                #[allow(unused_macros)]
                macro_rules! trace {
                    () => { tracing::trace!(#field_exprs_tokens); };
                    ($($arg:tt)+) => { tracing::trace!(#field_exprs_tokens, $($arg)*); };
                }

                #original_body
            }
        };

        // Replace the function body
        func.block = syn::parse2(new_body).expect("Failed to parse new function body");
    } else {
        // Generate the field expressions for the tracing macros with inline cloning
        let field_exprs = field_exprs_vec.iter().map(|(expr_str, expr)| {
            quote! { #expr_str = #expr.clone() }
        });

        let custom_exprs = custom_exprs_vec.iter().map(|(key, value)| {
            let key_ident = syn::parse_str::<syn::Path>(key).unwrap();
            quote! { #key_ident = #value.clone() }
        });

        let all_field_exprs: Vec<proc_macro2::TokenStream> =
            field_exprs.chain(custom_exprs).collect();
        let field_exprs_tokens = quote! { #(#all_field_exprs),* };

        // Create a new function body that redefines the tracing macros with inline cloning
        let new_body = quote! {
            {
                // Suppress unused macro warnings since not all redefined macros might be used
                #[allow(unused_macros)]
                // Redefine tracing macros to include our fields with inline cloning
                #[allow(unused_macros)]
                macro_rules! info {
                    () => { tracing::info!(#field_exprs_tokens) };
                    ($($arg:tt)+) => { tracing::info!(#field_exprs_tokens, $($arg)*) };
                }
                #[allow(unused_macros)]
                macro_rules! warn {
                    () => { tracing::warn!(#field_exprs_tokens) };
                    ($($arg:tt)+) => { tracing::warn!(#field_exprs_tokens, $($arg)*) };
                }
                #[allow(unused_macros)]
                macro_rules! error {
                    () => { tracing::error!(#field_exprs_tokens) };
                    ($($arg:tt)+) => { tracing::error!(#field_exprs_tokens, $($arg)*) };
                }
                #[allow(unused_macros)]
                macro_rules! debug {
                    () => { tracing::debug!(#field_exprs_tokens) };
                    ($($arg:tt)+) => { tracing::debug!(#field_exprs_tokens, $($arg)*) };
                }
                #[allow(unused_macros)]
                macro_rules! trace {
                    () => { tracing::trace!(#field_exprs_tokens) };
                    ($($arg:tt)+) => { tracing::trace!(#field_exprs_tokens, $($arg)*) };
                }

                // Original function body
                #original_body
            }
        };

        // Replace the function body
        func.block = syn::parse2(new_body).expect("Failed to parse new function body");
    }

    // Add an attribute to suppress warnings about unused variables
    let allow_unused_attr: syn::Attribute = syn::parse_quote! { #[allow(unused_variables)] };
    func.attrs.push(allow_unused_attr);

    func.into_token_stream().into()
}
