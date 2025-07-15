//! # log-args: Procedural Macro for Logging Function Arguments
//!
//! This crate provides the `#[params]` attribute macro to automatically log function arguments using the `tracing` crate.
//!
//! ## Features
//! - Log all function arguments by default
//! - Select specific arguments or fields to log
//! - Add custom key-value pairs to log output
//! - No tracing spans or span-related features used
//! - Compatible with both sync and async functions including tokio::spawn
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
//! - `no_fields`: Disable automatic field logging (useful with async move blocks that capture fields).
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
//! - Use `no_fields` with functions containing `async move` blocks to avoid lifetime issues.
//!
//! See the README and examples for more details.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::visit_mut::{self, VisitMut};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, ItemFn, Macro, Meta, MetaNameValue, Pat, Token};

struct LogArgs {
    fields: Vec<Expr>,
    custom: Vec<MetaNameValue>,
}

impl Parse for LogArgs {
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
                        fields.extend(list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?);
                    } else if list.path.is_ident("custom") {
                        custom.extend(list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?);
                    } else {
                        return Err(syn::Error::new_spanned(list.path, "Unknown attribute"));
                    }
                }
                _ => return Err(syn::Error::new_spanned(meta, "Unsupported attribute format")),
            }
        }
        Ok(LogArgs { fields, custom })
    }
}

struct InjectFieldsVisitor {
    log_fields: proc_macro2::TokenStream,
}

impl VisitMut for InjectFieldsVisitor {
    fn visit_macro_mut(&mut self, i: &mut Macro) {
        let is_log_macro = ["info", "warn", "error", "debug", "trace"]
            .iter()
            .any(|m| i.path.is_ident(m));

        if is_log_macro {
            let original_tokens = i.tokens.clone();
            let log_fields = &self.log_fields;

            if original_tokens.is_empty() {
                 i.tokens = quote! { #log_fields };
            } else {
                 i.tokens = quote! { #log_fields, #original_tokens };
            }
        }

        visit_mut::visit_macro_mut(self, i);
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

    let field_exprs = if params_args.fields.is_empty() {
        func.sig.inputs.iter().filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    if pat_ident.ident != "self" {
                        let ident_expr: Expr = syn::parse_str(&pat_ident.ident.to_string()).unwrap();
                        return Some(ident_expr);
                    }
                }
            }
            None
        }).collect::<Vec<_>>()
    } else {
        params_args.fields
    };

    let log_fields = field_exprs.iter().map(|expr| {
        let expr_str = expr.to_token_stream().to_string().replace(' ', "");
        quote! { #expr_str = ?(#expr).clone() }
    });

    let custom_fields = params_args.custom.iter().map(|nv| {
        let key = &nv.path;
        let value = &nv.value;
        quote! { #key = ?(#value).clone() }
    });

    let all_log_fields: Vec<proc_macro2::TokenStream> = log_fields.chain(custom_fields).collect();

    let mut visitor = InjectFieldsVisitor { log_fields: quote! { #(#all_log_fields),* } };
    visitor.visit_block_mut(&mut func.block);

    func.into_token_stream().into()
}
