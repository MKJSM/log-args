//! # log_args
//!
//! A simple procedural macro to log function arguments using the `tracing` crate.
//!
//! This crate provides a procedural macro attribute `#[log_args]` that can be applied to functions
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
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, ItemFn, Meta, MetaNameValue, Token};

/// A struct to parse the arguments passed to the `log_args` macro.
///
/// It supports two types of arguments:
/// - `fields(...)`: A list of expressions to be logged.
/// - `custom(...)`: A list of key-value pairs to be added to the log.
struct LogArgs {
    fields: Vec<Expr>,
    custom: Vec<MetaNameValue>,
}

impl Parse for LogArgs {
    /// Parses the token stream from the macro attribute into a `LogArgs` struct.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Vec::new();
        let mut custom = Vec::new();

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            match meta {
                Meta::List(list) => {
                    if list.path.is_ident("fields") {
                        let nested =
                            list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                        fields.extend(nested);
                    } else if list.path.is_ident("custom") {
                        let nested = list.parse_args_with(
                            Punctuated::<MetaNameValue, Token![,]>::parse_terminated,
                        )?;
                        custom.extend(nested);
                    } else {
                        return Err(syn::Error::new_spanned(
                            list.path,
                            "Unknown attribute, expected `fields` or `custom`",
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "Unsupported attribute format, expected `fields(...)` or `custom(...)`",
                    ))
                }
            }
        }

        Ok(LogArgs { fields, custom })
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
/// use log_args::log_args;
/// use tracing::info;
///
/// #[derive(Debug)]
/// struct User { id: u32 }
///
/// #[log_args]
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
/// use log_args::log_args;
/// use tracing::info;
///
/// #[derive(Debug)]
/// struct User { id: u32, name: String }
///
/// #[log_args(fields(user.id), custom(service = "auth"))]
/// fn authenticate(user: User) {
///     info!("Authenticating user");
/// }
///
/// // When called, this will produce a log similar to:
/// // INFO Authenticating user user_id=42 service="auth"
/// ```
#[proc_macro_attribute]
pub fn log_args(args: TokenStream, input: TokenStream) -> TokenStream {
    let log_args = syn::parse_macro_input!(args as LogArgs);
    let mut func = syn::parse_macro_input!(input as ItemFn);

    let mut fields_to_log = vec![];

    if log_args.fields.is_empty() && log_args.custom.is_empty() {
        for arg in &func.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let ident = &pat_ident.ident;
                    fields_to_log.push(quote! { #ident });
                }
            }
        }
    } else {
        for path in log_args.fields {
            fields_to_log.push(quote! { #path });
        }
    }

    let log_fields: Vec<_> = fields_to_log
        .iter()
        .map(|field| {
            let field_str = field.to_string().replace(' ', "");
            let key = syn::Ident::new(&field_str.replace('.', "_"), proc_macro2::Span::call_site());
            quote! { #key = ?#field }
        })
        .collect();

    let custom_fields: Vec<_> = log_args
        .custom
        .iter()
        .map(|nv| {
            let key = &nv.path;
            let val = &nv.value;
            quote! { #key = #val }
        })
        .collect();

    let stmts = &func.block.stmts;
    let tracing_block = quote! {
        {
            #[allow(unused_macros)]
            {
                macro_rules! info {
                    ($($t:tt)*) => {
                        tracing::info!(#(#log_fields,)* #(#custom_fields,)* $($t)*)
                    };
                }
                macro_rules! warn {
                    ($($t:tt)*) => {
                        tracing::warn!(#(#log_fields,)* #(#custom_fields,)* $($t)*)
                    };
                }
                macro_rules! error {
                    ($($t:tt)*) => {
                        tracing::error!(#(#log_fields,)* #(#custom_fields,)* $($t)*)
                    };
                }
                macro_rules! debug {
                    ($($t:tt)*) => {
                        tracing::debug!(#(#log_fields,)* #(#custom_fields,)* $($t)*)
                    };
                }

                #(#stmts)*
            }
        }
    };

    func.block = syn::parse2(tracing_block).expect("Failed to parse generated code");

    TokenStream::from(quote! { #func })
}
