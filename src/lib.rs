//! # log-args: Procedural Macro for Logging Function Arguments
//!
//! This crate provides the `#[params]` attribute macro to automatically log function arguments using the `tracing` crate.
//!
//! ## Features
//! - Log all function arguments by default
//! - Select specific arguments or fields to log
//! - Add custom key-value pairs to log output
//! - No tracing spans or span-related features used
//! - Compatible with both sync and async functions
//!
//! ## Usage
//!
//! ```rust
//! use log_args::params;
//! use tracing::info;
//!
//! #[derive(Debug)]
//! struct User { id: u32, name: String }
//!
//! #[params]
//! fn foo(user: User, count: usize) {
//!     info!("Function called");
//! }
//! ```
//!
//! ## Macro Attributes
//! - `fields(...)`: Log only the specified arguments or fields (e.g., `fields(user.id, count)`).
//! - `custom(...)`: Add custom key-value pairs to the log output (e.g., `custom(service = "auth")`).
//!
//! ## Example
//!
//! ```rust
//! #[params(fields(user.id), custom(service = "auth"))]
//! fn login(user: User) {
//!     info!("User login");
//! }
//! ```
//!
//! ## Limitations
//! - Only works with the `tracing` crate macros (`info!`, `debug!`, `warn!`, `error!`, `trace!`).
//! - Does not support span creation or level selection via macro input.
//! - All arguments must implement `Debug` for structured logging.
//!
//! See the README and examples for more details.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{spanned::Spanned, Expr, ItemFn, Meta, MetaNameValue, Pat, Token};

/// Parsed arguments for the `#[params]` macro.
///
/// - `fields`: List of expressions (arguments or fields) to log.
/// - `custom`: List of custom key-value pairs to add to the log.
struct LogArgs {
    fields: Vec<Expr>,
    custom: Vec<MetaNameValue>,
}

impl Parse for LogArgs {
    /// Parses the `fields(...)` and `custom(...)` attributes for the macro.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Vec::new();
        let mut custom = Vec::new();

        if input.is_empty() {
            return Ok(LogArgs { fields, custom });
        }

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            match meta {
                Meta::List(list) => {
                    if list.path.is_ident("fields") {
                        let nested = list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                        fields.extend(nested);
                    } else if list.path.is_ident("custom") {
                        let nested = list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
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
                    ));
                }
            }
        }

        Ok(LogArgs { fields, custom })
    }
}

/// Procedural macro to log function arguments using the `tracing` macros.
///
/// # Usage
///
/// ```rust
/// #[params]
/// fn foo(arg1: i32, arg2: String) {
///     info!("Called foo");
/// }
///
/// #[params(fields(arg1), custom(service = "api"))]
/// fn bar(arg1: i32, arg2: String) {
///     info!("Called bar");
/// }
/// ```
///
/// - Use `fields(...)` to select which arguments/fields to log.
/// - Use `custom(...)` to add custom key-value pairs.
///
/// Only `tracing` macros are supported. No spans are created.
#[proc_macro_attribute]
pub fn params(args: TokenStream, input: TokenStream) -> TokenStream {
    let params: LogArgs = match syn::parse(args) {
        Ok(params) => params,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut func: ItemFn = match syn::parse(input) {
        Ok(func) => func,
        Err(e) => return e.to_compile_error().into(),
    };

    let log_fields: Vec<_> = if params.fields.is_empty() {
        func.sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let ident = &pat_ident.ident;
                        return Some(quote! { #ident = ?#ident });
                    }
                }
                None
            })
            .collect()
    } else {
        params
            .fields
            .iter()
            .map(|expr| {
                let expr_str = quote!(#expr).to_string().replace(' ', "").replace('.', "_");
                let key = syn::Ident::new(&expr_str, expr.span());
                quote! { #key = ?#expr }
            })
            .collect()
    };

    let custom_fields: Vec<_> = params
        .custom
        .iter()
        .map(|nv| {
            let key = &nv.path;
            let val = &nv.value;
            quote! { #key = #val }
        })
        .collect();

    func.attrs.push(syn::parse_quote! { #[allow(unused_macros)] });

    let stmts = &func.block.stmts;

    let tracing_block = quote! {
        {
            macro_rules! info {
                ($($t:tt)*) => { tracing::info!(#(#log_fields,)* #(#custom_fields,)* $($t)*) };
            }
            macro_rules! warn {
                ($($t:tt)*) => { tracing::warn!(#(#log_fields,)* #(#custom_fields,)* $($t)*) };
            }
            macro_rules! error {
                ($($t:tt)*) => { tracing::error!(#(#log_fields,)* #(#custom_fields,)* $($t)*) };
            }
            macro_rules! debug {
                ($($t:tt)*) => { tracing::debug!(#(#log_fields,)* #(#custom_fields,)* $($t)*) };
            }
            macro_rules! trace {
                ($($t:tt)*) => { tracing::trace!(#(#log_fields,)* #(#custom_fields,)* $($t)*) };
            }

            #(#stmts)*
        }
    };

    func.block = match syn::parse2(tracing_block) {
        Ok(block) => block,
        Err(e) => return e.to_compile_error().into(),
    };

    TokenStream::from(quote! { #func })
}
