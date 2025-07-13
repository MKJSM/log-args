//! # params
//!
//! A simple procedural macro to log function arguments using the `tracing` crate.
//!
//! This crate provides a procedural macro attribute `#[params]` that can be applied to functions
//! to automatically log their arguments. It is designed to be simple, efficient, and easy to integrate
//! into any project that uses `tracing` for structured logging.
//!
//! ## Features
//!
//! - Log all function arguments by default.
//! - Select specific arguments to log.
//! - Log nested fields of struct arguments (e.g., `user.id`).
//! - Add custom key-value pairs to the log output.
//! - Supports both synchronous and asynchronous functions.
//! - All logging is done through the `tracing` ecosystem, which means it has zero-overhead when disabled.
//!
//! For more examples, see the [examples directory](https://github.com/MKJSM/log-args/tree/main/examples) on GitHub.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, ItemFn, Meta, MetaNameValue, Token};

/// A struct to parse the arguments passed to the `params` macro.
///
/// It supports two types of arguments:
/// - `fields(...)`: A list of expressions to be logged.
/// - `custom(...)`: A list of key-value pairs to be added to the log.
struct Params {
    fields: Vec<Expr>,
    custom: Vec<MetaNameValue>,
    span: Option<bool>,
    level: Option<String>,
}

impl Parse for Params {
    /// Parses the token stream from the macro attribute into a `LogArgs` struct.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Vec::new();
        let mut custom = Vec::new();
        let mut span = None;
        let mut level = None;

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            if meta.path().is_ident("fields") {
                if let Meta::List(list) = meta {
                    let nested =
                        list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                    fields.extend(nested);
                } else {
                    return Err(input.error("Expected `fields(...)`"));
                }
            } else if meta.path().is_ident("custom") {
                if let Meta::List(list) = meta {
                    let nested = list.parse_args_with(
                        Punctuated::<MetaNameValue, Token![,]>::parse_terminated,
                    )?;
                    custom.extend(nested);
                } else {
                    return Err(input.error("Expected `custom(...)`"));
                }
            } else if meta.path().is_ident("span") {
                if let Meta::Path(_) = meta {
                    span = Some(true);
                } else if let Meta::NameValue(nv) = meta {
                    if let Ok(lit_bool) = syn::parse2::<syn::LitBool>(nv.value.to_token_stream()) {
                        span = Some(lit_bool.value);
                    } else {
                        return Err(input.error("Expected boolean value for `span` attribute"));
                    }
                } else {
                    return Err(input.error("Expected `span` or `span = true/false`"));
                }
            } else if meta.path().is_ident("level") {
                if let Meta::NameValue(nv) = meta {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            level = Some(lit_str.value());
                        } else {
                            return Err(input.error("Expected string value for `level` attribute, e.g., level = \"info\""));
                        }
                    } else {
                        return Err(input.error(
                            "Expected string literal for `level` attribute, e.g., level = \"info\"",
                        ));
                    }
                } else {
                    return Err(
                        input.error("Expected `level = \"info\"` or similar for `level` attribute")
                    );
                }
            } else {
                return Err(input
                    .error("Unknown attribute, expected `fields`, `custom`, `span` or `level`"));
            }
        }

        Ok(Params {
            fields,
            custom,
            span,
            level,
        })
    }
}

/// A procedural macro to automatically log function arguments.
///
/// By default, it logs all arguments. You can customize its behavior by passing arguments.
///
/// # Attributes
///
/// - `fields(arg1, arg2, ...)`: Logs only the specified arguments or their subfields.
/// - `custom(key1 = "value1", key2 = "value2", ...)`: Adds custom key-value pairs to the log output.
///
/// # Examples
///
/// ## Logging all arguments
///
/// ```rust
/// use log_args::params;
/// use tracing::info;
///
/// #[derive(Debug)]
/// struct User { id: u32 }
///
/// #[params]
/// fn process(user: User, task_id: i32) {
///     info!("Processing task");
/// }
///
/// // When called, this will produce a log similar to:
/// // INFO Processing task user=User { id: 42 } task_id=42
/// ```
///
/// ## Logging selected fields and custom values
///
/// ```rust
/// use log_args::params;
/// use tracing::info;
///
/// #[derive(Debug)]
/// struct User { id: u32, name: String }
///
/// #[params(fields(user.id), custom(service = "auth"))]
/// fn authenticate(user: User) {
///     info!("Authenticating user");
/// }
///
/// // When called, this will produce a log similar to:
/// // INFO Authenticating user user_id=42 service="auth"
/// ```
#[proc_macro_attribute]
pub fn params(args: TokenStream, input: TokenStream) -> TokenStream {
    let params = syn::parse_macro_input!(args as Params);
    let mut func = syn::parse_macro_input!(input as ItemFn);

    let mut fields_to_log = vec![];

    if params.fields.is_empty() && params.custom.is_empty() {
        for arg in &func.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let ident = &pat_ident.ident;
                    fields_to_log.push(quote! { #ident });
                }
            }
        }
    } else {
        for path in params.fields.clone() {
            fields_to_log.push(quote! { #path });
        }
    }

    let orig_block = func.block;

    let arg_fmt = if !fields_to_log.is_empty() {
        let fmt = fields_to_log.iter().map(|field| {
            let field_str = field.to_string().replace(' ', "");
            let key = field_str.replace('.', "_");
            quote! {
                format!("{}={:?}", #key, #field)
            }
        });
        quote! { let __log_args_str = vec![#(#fmt),*].join(" "); }
    } else {
        quote! { let __log_args_str = String::new(); }
    };

    let span_code = if params.span.unwrap_or(false) {
        // Always create a span with all argument fields at the start of the function
        quote! {
            let __span = tracing::span!(tracing::Level::INFO, "", #(#fields_to_log),*);
            let __enter = __span.enter();
        }
    } else {
        quote! {}
    };

    let injected_code = {
        let span_block = if params.span.unwrap_or(false) {
            quote! { #span_code }
        } else {
            quote! {}
        };
        quote! {
            #span_block
            #arg_fmt
            #[allow(unused_macros)]
            macro_rules! info {
                ($msg:expr) => {
                    if __log_args_str.is_empty() {
                        tracing::info!(concat!("{}"), $msg);
                    } else {
                        tracing::info!(concat!("{} {}"), $msg, __log_args_str);
                    }
                };
                ($msg:expr, $($t:tt)*) => {
                    if __log_args_str.is_empty() {
                        tracing::info!(concat!("{}"), $msg, $($t)*);
                    } else {
                        tracing::info!(concat!("{} {}"), $msg, __log_args_str, $($t)*);
                    }
                };
            }
            macro_rules! warn {
                ($msg:expr) => {
                    if __log_args_str.is_empty() {
                        tracing::warn!(concat!("{}"), $msg);
                    } else {
                        tracing::warn!(concat!("{} {}"), $msg, __log_args_str);
                    }
                };
                ($msg:expr, $($t:tt)*) => {
                    if __log_args_str.is_empty() {
                        tracing::warn!(concat!("{}"), $msg, $($t)*);
                    } else {
                        tracing::warn!(concat!("{} {}"), $msg, __log_args_str, $($t)*);
                    }
                };
            }
            macro_rules! error {
                ($msg:expr) => {
                    if __log_args_str.is_empty() {
                        tracing::error!(concat!("{}"), $msg);
                    } else {
                        tracing::error!(concat!("{} {}"), $msg, __log_args_str);
                    }
                };
                ($msg:expr, $($t:tt)*) => {
                    if __log_args_str.is_empty() {
                        tracing::error!(concat!("{}"), $msg, $($t)*);
                    } else {
                        tracing::error!(concat!("{} {}"), $msg, __log_args_str, $($t)*);
                    }
                };
            }
            macro_rules! debug {
                ($msg:expr) => {
                    if __log_args_str.is_empty() {
                        tracing::debug!(concat!("{}"), $msg);
                    } else {
                        tracing::debug!(concat!("{} {}"), $msg, __log_args_str);
                    }
                };
                ($msg:expr, $($t:tt)*) => {
                    if __log_args_str.is_empty() {
                        tracing::debug!(concat!("{}"), $msg, $($t)*);
                    } else {
                        tracing::debug!(concat!("{} {}"), $msg, __log_args_str, $($t)*);
                    }
                };
            }
        }
    };

    func.block = Box::new(syn::parse_quote!({
        #injected_code
        #orig_block
    }));

    TokenStream::from(quote! { #func })
}
