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
//! - Supports `Debug` (`:?`) and `Display` (`{}`) formatting for fields.
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
//! // Log all arguments (will use Debug format for `User` and `count`)
//! #[params]
//! fn process_user(user: User, count: usize) {
//!     info!("Processing user data");
//!     // Example log output (format may vary based on tracing-subscriber):
//!     // INFO process_user: Processing user data user=User { id: 42, name: "Alice" } count=5
//! }
//!
//! // Log only specific fields
//! #[params(fields(user.id, count))]
//! fn validate_user(user: User, count: usize) {
//!     info!("Validating user");
//!     // Example log output:
//!     // INFO validate_user: Validating user user.id=42 count=5
//! }
//!
//! // Add custom values
//! #[params(custom(service = "auth", version = "1.0"))]
//! fn authenticate(user: User) {
//!     info!("Authentication attempt");
//!     // Example log output:
//!     // INFO authenticate: Authentication attempt user=User { id: 42, name: "Alice" } service="auth" version="1.0"
//! }
//!
//! // Log specific fields with mixed Debug and Display formatting
//! #[params(fields(user.id, user.name), display_fields(user.name))]
//! fn log_user_details(user: User) {
//!     info!("Logging user details");
//!     // Example log output:
//!     // INFO log_user_details: Logging user details user.id=42 user.name=Alice
//!     // Note: "Alice" is not quoted because `display_fields(user.name)` was used.
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
//!     // The `client` variable is still available here for logging
//!     // even if it's moved into another async block later.
//!     
//!     let task_client_id = client.id.clone();
//!     let task = tokio::spawn(async move {
//!         // Use task_client_id here without ownership issues
//!         info!(client_id = ?task_client_id, "Worker task for client started");
//!     });
//!     
//!     // Logs still work even though parts of `client` were conceptually "moved"
//!     // because values were cloned upfront and stored in thread-locals for logging.
//!     info!("Waiting for client processing to complete");
//!     task.await.unwrap();
//! }
//! ```
//!
//! ## Macro Attributes
//!
//! - `fields(...)`: Specifies arguments or fields to include in log output (e.g., `fields(user.id, count)`).
//!   These fields will use `Debug` (`:?`) formatting by default, unless also listed in `display_fields`.
//! - `display_fields(...)`: Specifies arguments or fields that should use `Display` (`{}`) formatting.
//!   This is ideal for `String`, `&str`, numeric types, `bool`, or any type implementing `Display` where you
//!   want the "plain" output without `Debug`'s extra quotes or structural output.
//! - `custom(...)`: Adds custom key-value pairs to the log output (e.g., `custom(service = "auth")`).
//!   Custom values will use `Display` (`{}`) formatting.
//! - `clone_upfront`: If present, clones all specified fields at the beginning of the function and stores
//!   their formatted `String` representations in thread-local storage. This is crucial for async functions
//!   where argument ownership might change (e.g., when moving into `async move` blocks or `tokio::spawn`),
//!   ensuring the logger always has access to the values.
//!
//! ## How It Works
//!
//! The `#[params]` macro transforms your function by redefining the `tracing` macros (`info!`, `warn!`,
//! `error!`, `debug!`, `trace!`) within its scope. These redefined macros automatically inject the specified
//! function arguments or fields into every log call.
//!
//! - **Without `clone_upfront`**: The arguments are cloned inline at each `tracing` macro call.
//! - **With `clone_upfront`**: At the very beginning of the function, all specified fields are cloned,
//!   formatted (either `Debug` or `Display` based on `display_fields`), and stored as `String`s in
//!   thread-local variables. The redefined `tracing` macros then read these `String`s from the thread-locals.
//!   This ensures that the logging mechanism does not interfere with the original argument's ownership
//!   flow within an `async` function, making them available for logging throughout the function's
//!   entire execution, even if the original values are moved or dropped.
//!
//! ## Limitations
//! - Only works with the `tracing` crate macros (`info!`, `debug!`, `warn!`, `error!`, `trace!`).
//! - Does not support span creation or level selection via macro input.
//! - All arguments must implement `Clone` and either `Debug` (default) or `Display` (if specified via `display_fields`)
//!   for their values to be captured.
//! - Generates warnings about unused macro definitions (`#[allow(unused_macros)]` is added to suppress these).
//!
//! See the `examples` directory for more detailed usage patterns.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token,
};

struct LogArgs {
    fields: Vec<Expr>,
    custom: Vec<MetaNameValue>,
    display_fields: Vec<Expr>,
    clone_upfront: bool,
    span: bool,
}

impl Parse for LogArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Vec::new();
        let mut custom = Vec::new();
        let mut display_fields = Vec::new();
        let mut clone_upfront = false;
        let mut span = false;

        if input.is_empty() {
            return Ok(LogArgs {
                fields,
                custom,
                display_fields,
                clone_upfront,
                span,
            });
        }

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
                    } else if list.path.is_ident("display_fields") {
                        display_fields.extend(
                            list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?,
                        );
                    } else {
                        return Err(syn::Error::new_spanned(
                            list.path,
                            "Unknown attribute, expected `fields`, `custom`, or `display_fields`",
                        ));
                    }
                }
                Meta::Path(path) => {
                    if path.is_ident("clone_upfront") {
                        clone_upfront = true;
                    } else if path.is_ident("span") {
                        span = true;
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "Unknown attribute, expected `clone_upfront` or `span`",
                        ));
                    }
                }
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("clone_upfront") {
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
                    } else if name_value.path.is_ident("span") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Bool(lit_bool),
                            ..
                        }) = &name_value.value
                        {
                            span = lit_bool.value;
                        } else {
                            return Err(syn::Error::new_spanned(
                                name_value.value,
                                "Expected a boolean literal for `span`",
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
            display_fields,
            clone_upfront,
            span,
        })
    }
}

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

    if params_args.fields.is_empty() && params_args.custom.is_empty() {
        return func.into_token_stream().into();
    }

    let display_field_strings: HashSet<String> = params_args
        .display_fields
        .iter()
        .map(|expr| expr.to_token_stream().to_string().replace(' ', ""))
        .collect();

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

    let original_body = func.block;

    // Get function name for span creation
    let fn_name = format!("{}", func.sig.ident);
    let fn_id = syn::Ident::new(&fn_name, proc_macro2::Span::call_site());
    
    if params_args.clone_upfront {
        // fn_name and fn_id are already defined above

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

        let tls_inits = tls_var_names.iter().map(|(expr_str, tls_var)| {
            let expr_index = field_exprs_vec
                .iter()
                .position(|(s, _)| s == expr_str)
                .unwrap();
            let (_orig_expr_str, expr_to_capture) = &field_exprs_vec[expr_index]; // Use _orig_expr_str to avoid unused warning

            let format_specifier = if display_field_strings.contains(expr_str) {
                "{}" // Display format
            } else {
                "{:?}" // Debug format
            };

            // Use syn::parse_str to create a TokenStream from the formatted string
            // This is the crucial part that was fixed.
            let format_macro_tokens = syn::parse_str::<proc_macro2::TokenStream>(&format!(
                "format!(\"{}\", &#expr_to_capture)",
                format_specifier
            ))
            .expect("Failed to parse format! macro tokens for thread_local init");

            quote! {
                #tls_var.with(|cell| {
                    *cell.borrow_mut() = Some(#format_macro_tokens);
                });
            }
        });

        let custom_tls_inits = custom_tls_var_names.iter().map(|(key, tls_var)| {
            let custom_index = custom_exprs_vec.iter().position(|(k, _)| k == key).unwrap();
            let (_orig_key, value_to_capture) = &custom_exprs_vec[custom_index]; // Use _orig_key

            quote! {
                #tls_var.with(|cell| {
                    *cell.borrow_mut() = Some(format!("{}", &#value_to_capture));
                });
            }
        });

        let field_exprs = tls_var_names.iter().map(|(expr_str, tls_var)| {
            let key_parts: Vec<&str> = expr_str.split('.').collect();
            let key = key_parts.last().copied().unwrap_or(expr_str.as_str());

            quote! { #key = #tls_var.with(|cell| cell.borrow().clone().unwrap_or_default()) }
        });

        let custom_exprs = custom_tls_var_names.iter().map(|(key, tls_var)| {
            let key_ident = syn::parse_str::<syn::Path>(key).unwrap();

            quote! { #key_ident = #tls_var.with(|cell| cell.borrow().clone().unwrap_or_default()) }
        });

        let all_field_exprs: Vec<proc_macro2::TokenStream> =
            field_exprs.chain(custom_exprs).collect();
        let field_exprs_tokens = quote! { #(#all_field_exprs),* };

        let new_body = if params_args.span {
            quote! {
                {
                    #[allow(unused_macros)]
                    #(#thread_locals)*
                    #(#custom_thread_locals)*
    
                    #(#tls_inits)*
                    #(#custom_tls_inits)*
    
                    let __log_args_span = tracing::span!(tracing::Level::INFO, #fn_name, #field_exprs_tokens);
                    let __log_args_span_guard = __log_args_span.enter();

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
            }
        } else {
            quote! {
                {
                    #[allow(unused_macros)]
                    #(#thread_locals)*
                    #(#custom_thread_locals)*
    
                    #(#tls_inits)*
                    #(#custom_tls_inits)*
    
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
            }
        };

        func.block = syn::parse2(new_body).expect("Failed to parse new function body");
    } else {
        let field_exprs = field_exprs_vec.iter().map(|(expr_str, expr)| {
            let formatter = if display_field_strings.contains(expr_str) {
                ""
            } else {
                "?"
            };
            let formatter_tokens: proc_macro2::TokenStream = formatter.parse().unwrap();

            quote! { #expr_str = #formatter_tokens #expr.clone() }
        });

        let custom_exprs = custom_exprs_vec.iter().map(|(key, value)| {
            let key_ident = syn::parse_str::<syn::Path>(key).unwrap();
            quote! { #key_ident = #value.clone() }
        });

        let all_field_exprs: Vec<proc_macro2::TokenStream> =
            field_exprs.chain(custom_exprs).collect();
        let field_exprs_tokens = quote! { #(#all_field_exprs),* };

        let new_body = if params_args.span {
            quote! {
                {
                    let __log_args_span = tracing::span!(tracing::Level::INFO, #fn_name, #field_exprs_tokens);
                    let __log_args_span_guard = __log_args_span.enter();

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

                    #original_body
                }
            }
        } else {
            quote! {
                {
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

                    #original_body
                }
            }
        };

        func.block = syn::parse2(new_body).expect("Failed to parse new function body");
    }

    let allow_unused_attr: syn::Attribute = syn::parse_quote! { #[allow(unused_variables)] };
    func.attrs.push(allow_unused_attr);

    func.into_token_stream().into()
}
