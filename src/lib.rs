//! # log-args
//!
//! A procedural macro to automatically log function arguments using the [`tracing`](https://crates.io/crates/tracing) crate.
//!
//! ## Overview
//!
//! This crate provides the `#[params]` attribute macro, which can be applied to functions to automatically log their arguments using the `tracing` ecosystem. It is designed to be simple, efficient, and easy to integrate into any project that uses `tracing` for structured logging.
//!
//! ## Features
//!
//! - Log all function arguments by default.
//! - Select specific arguments to log with `fields(...)`.
//! - Log nested fields of struct arguments (e.g., `user.id`).
//! - Supports both synchronous and asynchronous functions.
//! - All logging is done through `tracing`, with zero-overhead when disabled.
//! - Compile-time validation for macro attributes.
//!
//! ## Example Usage
//!
//! ```rust
//! use log_args::params;
//! use tracing::info;
//!
//! #[derive(Debug)]
//! struct User { id: u32 }
//!
//! #[params]
//! fn process_user(user: User, task_id: i32) {
//!     info!("Processing task");
//! }
//! // Output: INFO Processing task user=User { id: 42 } task_id=100
//! ```
//!
//! ### Log Specific Fields
//!
//! ```rust
//! use log_args::params;
//! #[derive(Debug)]
//! struct User { id: u32 }
//!
//! #[params(fields(user.id))]
//! fn process_user(user: User) {
//!     // ...
//! }
//! // Output: ... user_id=42
//! ```
//!
//! ### Async and Span Support
//!
//! ```rust
//! use log_args::params;
//! #[allow(unused)]
//! #[params]
//! async fn my_async_fn(arg: i32) {
//!     // ...
//! }
//! // Output: logs are scoped to a tracing span
//! ```
//!
//! ## Attribute Options
//!
//! - `fields(arg1, arg2, ...)`: Logs only the specified arguments or their subfields.
//! - `span = true/false`: If true, wraps the function in a tracing span (default: false). This attribute is currently ignored but reserved for future use.
//!
//! ## Limitations
//!
//! - Logging context is local to the annotated function. Subfunctions do not inherit logged fields.
//! - Deep field expressions (e.g., `user.name.first`) are not yet supported.
//!
//! ## More Examples
//!
//! See the [examples directory](https://github.com/MKJSM/log-args/tree/main/examples) on GitHub.
//!
//! ## Runtime
//!
//! This macro requires the companion crate [`log-args-runtime`](https://crates.io/crates/log-args-runtime) for runtime context propagation. This is handled automatically if you depend on `log-args`.
//!
//! ## License
//!
//! Licensed under MIT or Apache-2.0. See [LICENSE](https://github.com/MKJSM/log-args/blob/main/LICENSE).

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, ItemFn, Meta, Token};

/// A struct to parse the arguments passed to the `params` macro.
///
/// It supports one type of argument:
/// - `fields(...)`: A list of expressions to be logged.
struct Params {
    fields: Vec<Expr>,
}

impl Parse for Params {
    /// Parses the token stream from the macro attribute into a `Params` struct.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields_to_log: Punctuated<Expr, Token![,]> = Punctuated::new();

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            if meta.path().is_ident("fields") {
                if let Meta::List(list) = meta {
                    let nested =
                        list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                    fields_to_log.extend(nested);
                } else {
                    return Err(input.error("Expected `fields(...)`"));
                }
            } else {
                let path = meta
                    .path()
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                if path != "span" {
                    // We ignore `span` but don't error on it
                    return Err(
                        input.error(format!("Unknown attribute `{path}`, expected `fields`"))
                    );
                }
            }
        }

        Ok(Params {
            fields: fields_to_log.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn params(args: TokenStream, input: TokenStream) -> TokenStream {
    let params = syn::parse_macro_input!(args as Params);
    let mut func = syn::parse_macro_input!(input as ItemFn);

    let fields_to_log = if params.fields.is_empty() {
        func.sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let ident = &pat_ident.ident;
                        let expr: syn::Expr = syn::parse_str(&ident.to_string()).unwrap();
                        return Some(expr);
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    } else {
        params.fields
    };

    let fields_to_log_keys = fields_to_log
        .iter()
        .map(|expr| quote!(#expr).to_string().replace(' ', "").replace('.', "_"));

    let orig_block = func.block;

    let arg_fmt = if !fields_to_log.is_empty() {
        quote! {
            use std::collections::HashMap;
            let mut map = HashMap::new();

            let __log_args_parent = log_args_runtime::__PARENT_LOG_ARGS.with(|slot| slot.borrow().clone());
            if let Some(parent) = __log_args_parent.as_ref() {
                for kv_pair in parent.split_whitespace() {
                    if let Some((k, v)) = kv_pair.split_once('=') {
                        map.insert(k.to_string(), v.to_string());
                    }
                }
            }

            #(map.insert(#fields_to_log_keys.to_string(), format!("{:?}", #fields_to_log));)*

            let mut sorted_keys: Vec<_> = map.keys().collect();
            sorted_keys.sort();
            let __log_args_str = sorted_keys.iter()
                .map(|k| format!("{}={}", k, map.get(*k).unwrap()))
                .collect::<Vec<_>>().join(" ");
        }
    } else {
        quote! { let __log_args_str = log_args_runtime::__PARENT_LOG_ARGS.with(|slot| slot.borrow().clone().unwrap_or_default()); }
    };

    let fn_name_str = func.sig.ident.to_string();
    let pascal_fn_name = fn_name_str
        .split('_')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>();

    let span_code = {
        let __log_args_str_copy = quote! { let __log_args_str_copy = __log_args_str.clone(); };
        quote! {
            #__log_args_str_copy
            log_args_runtime::__PARENT_LOG_ARGS.with(|slot| {
                *slot.borrow_mut() = if __log_args_str_copy.is_empty() { None } else { Some(__log_args_str_copy) };
            });
        }
    };

    let injected_code = {
        quote! {
            #arg_fmt
            let __log_args_final = if __log_args_str.is_empty() { log_args_runtime::__PARENT_LOG_ARGS.with(|slot| slot.borrow().clone()).unwrap_or_default() } else { __log_args_str.clone() };

            #[allow(unused_macros)]
            macro_rules! info {
                ($($t:tt)*) => {
                    if __log_args_final.is_empty() {
                        tracing::info!("{}: {}", #pascal_fn_name, format_args!($($t)*));
                    } else {
                        tracing::info!("{}: {} {}", #pascal_fn_name, format_args!($($t)*), __log_args_final);
                    }
                };
            }
            #[allow(unused_macros)]
            macro_rules! warn {
                ($($t:tt)*) => {
                    if __log_args_final.is_empty() {
                        tracing::warn!("{}: {}", #pascal_fn_name, format_args!($($t)*));
                    } else {
                        tracing::warn!("{}: {} {}", #pascal_fn_name, format_args!($($t)*), __log_args_final);
                    }
                };
            }
            #[allow(unused_macros)]
            macro_rules! error {
                ($($t:tt)*) => {
                    if __log_args_final.is_empty() {
                        tracing::error!("{}: {}", #pascal_fn_name, format_args!($($t)*));
                    } else {
                        tracing::error!("{}: {} {}", #pascal_fn_name, format_args!($($t)*), __log_args_final);
                    }
                };
            }
            #[allow(unused_macros)]
            macro_rules! debug {
                ($($t:tt)*) => {
                    if __log_args_final.is_empty() {
                        tracing::debug!("{}: {}", #pascal_fn_name, format_args!($($t)*));
                    } else {
                        tracing::debug!("{}: {} {}", #pascal_fn_name, format_args!($($t)*), __log_args_final);
                    }
                };
            }
            #span_code
        }
    };

    func.block = Box::new(syn::parse_quote!({
        #injected_code
        #orig_block
    }));

    TokenStream::from(quote! { #func })
}
