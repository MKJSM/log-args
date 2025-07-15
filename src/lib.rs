extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, ItemFn, Lit, Meta, Pat};

// Define custom types for attribute parsing
#[derive(Debug)]
struct AttributeArgs {
    args: Vec<AttributeArg>,
}

#[derive(Debug)]
enum AttributeArg {
    Span,
    Fields(Vec<String>),
    Custom(Vec<(String, Lit)>),
    LogName(String),
    Unknown(String),
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();

        // If input is empty, return empty args
        if input.is_empty() {
            return Ok(AttributeArgs { args });
        }

        // Parse comma-separated meta items
        let metas = Punctuated::<Meta, Comma>::parse_terminated(input)?;

        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("span") => {
                    args.push(AttributeArg::Span);
                }
                Meta::List(list) if list.path.is_ident("fields") => {
                    let content = list.tokens;
                    let content_str = content.to_string();
                    let fields = content_str
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    args.push(AttributeArg::Fields(fields));
                }
                Meta::List(list) if list.path.is_ident("custom") => {
                    let content = list.tokens;
                    let content_str = content.to_string();
                    let pairs: Vec<(String, Lit)> = content_str
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .split(',')
                        .filter_map(|s| {
                            let parts: Vec<&str> = s.split('=').collect();
                            if parts.len() == 2 {
                                let key = parts[0].trim().to_string();
                                let value_str = parts[1].trim();
                                // Simple string literal parsing
                                if value_str.starts_with('"') && value_str.ends_with('"') {
                                    let value =
                                        value_str.trim_start_matches('"').trim_end_matches('"');
                                    Some((
                                        key,
                                        Lit::Str(syn::LitStr::new(
                                            value,
                                            proc_macro2::Span::call_site(),
                                        )),
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    args.push(AttributeArg::Custom(pairs));
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("log_name") => {
                    if let syn::Expr::Lit(expr_lit) = &name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            args.push(AttributeArg::LogName(lit_str.value()));
                        }
                    }
                }
                _ => {
                    // Unknown attribute - report it for better error messages
                    if let Some(ident) = meta.path().get_ident() {
                        args.push(AttributeArg::Unknown(ident.to_string()));
                    }
                }
            }
        }

        Ok(AttributeArgs { args })
    }
}

#[proc_macro_attribute]
pub fn params(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let mut func = parse_macro_input!(item as ItemFn);
    let fn_name = func.sig.ident.to_string();
    let orig_block = &func.block;

    let mut has_span = false;
    let mut field_exprs = Vec::new();
    let mut custom_fields = Vec::new();
    let mut log_name = None;
    let mut unknown_attrs = Vec::new();

    for arg in attr_args.args {
        match arg {
            AttributeArg::Span => {
                has_span = true;
            }
            AttributeArg::Fields(fields) => {
                for field in fields {
                    field_exprs.push(field);
                }
            }
            AttributeArg::Custom(pairs) => {
                for pair in pairs {
                    custom_fields.push(pair);
                }
            }
            AttributeArg::LogName(name) => {
                log_name = Some(name);
            }
            AttributeArg::Unknown(name) => {
                unknown_attrs.push(name);
            }
        }
    }

    // Return error for unknown attributes
    if !unknown_attrs.is_empty() {
        let error_msg = format!("unexpected end of input, Unknown attribute `{}`, expected `span`, `fields`, `custom`, or `log_name`", unknown_attrs.join(", "));
        return syn::Error::new(proc_macro2::Span::call_site(), error_msg)
            .to_compile_error()
            .into();
    }

    // Use custom log name if provided, otherwise use function name
    let display_name = log_name.as_ref().unwrap_or(&fn_name).clone();

    // Capitalize first letter of display name for better log readability
    let mut chars = display_name.chars();
    let _fn_name_display = match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    };

    // Use the display name for the function field and span name
    let fn_name_token = if let Some(name) = &log_name {
        name.clone()
    } else {
        fn_name.clone()
    };

    // Create field expressions based on field names or all function arguments
    let log_fields = if !field_exprs.is_empty() {
        // User specified `fields(...)`, so we only log those.
        let mut field_expr_tokens = Vec::new();

        for field_name in field_exprs.clone() {
            // Handle subfields with dot notation (e.g., user.id)
            if field_name.contains('.') {
                let parts: Vec<&str> = field_name.split('.').collect();
                if parts.len() == 2 {
                    let parent = parts[0];
                    let child = parts[1];
                    let display_name = format!("{}_{}" ,parent, child);
                    let parent_ident = syn::Ident::new(parent, proc_macro2::Span::call_site());
                    let child_ident = syn::Ident::new(child, proc_macro2::Span::call_site());
                    let display_ident =
                        syn::Ident::new(&display_name, proc_macro2::Span::call_site());
                    field_expr_tokens.push(quote! {
                        #display_ident = tracing::field::debug(&#parent_ident.#child_ident)
                    });
                }
            } else {
                // Simple field
                let ident = syn::Ident::new(&field_name, proc_macro2::Span::call_site());
                field_expr_tokens.push(quote! {
                    #ident = tracing::field::debug(&#ident)
                });
            }
        }
        field_expr_tokens
    } else {
        // No `fields` specified, so log all function arguments.
        func.sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let ident = &pat_ident.ident;
                        if !ident.to_string().starts_with('_') {
                            return Some(quote! {
                                #ident = tracing::field::debug(&#ident)
                            });
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    };

    // Process custom key-value pairs
    let custom_field_tokens = custom_fields
        .iter()
        .map(|(key, val)| {
            let key_ident = syn::Ident::new(key, proc_macro2::Span::call_site());
            quote! { #key_ident = #val }
        })
        .collect::<Vec<_>>();

    // Generate new block that redefines tracing macros
    let new_block = if has_span {
        quote! {
            {
                // Store function name and arguments for use in macros
                let __fn_name = #fn_name_token;
                
                // Redefine tracing macros to include our fields
                // Redefine tracing macros to include our fields
                macro_rules! trace {
                    ($msg:expr) => {
                        tracing::trace!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $msg
                        )
                    };
                    ($fmt:expr, $($arg:tt)*) => {
                        tracing::trace!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $fmt, $($arg)*
                        )
                    };
                }
                
                macro_rules! debug {
                    ($msg:expr) => {
                        tracing::debug!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $msg
                        )
                    };
                    ($fmt:expr, $($arg:tt)*) => {
                        tracing::debug!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $fmt, $($arg)*
                        )
                    };
                }
                
                macro_rules! info {
                    ($msg:expr) => {
                        tracing::info!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $msg
                        )
                    };
                    ($fmt:expr, $($arg:tt)*) => {
                        tracing::info!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $fmt, $($arg)*
                        )
                    };
                }
                
                macro_rules! warn {
                    ($msg:expr) => {
                        tracing::warn!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $msg
                        )
                    };
                    ($fmt:expr, $($arg:tt)*) => {
                        tracing::warn!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $fmt, $($arg)*
                        )
                    };
                }
                
                macro_rules! error {
                    ($msg:expr) => {
                        tracing::error!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $msg
                        )
                    };
                    ($fmt:expr, $($arg:tt)*) => {
                        tracing::error!(
                            function = __fn_name,
                            #(#log_fields,)*
                            #(#custom_field_tokens,)*
                            $fmt, $($arg)*
                        )
                    };
                }
                
                // Execute original function body with redefined macros
                #orig_block
            }
        }
    } else {
        // Without span, just log at the beginning and execute original block
        quote! {
            {
                // Log function arguments at the beginning
                tracing::debug!(
                    function = #fn_name_token,
                    #(#log_fields,)*
                    #(#custom_field_tokens,)*
                    "Function called"
                );
                
                // Execute original function body
                #orig_block
            }
        }
    };

    // Replace the function body with our instrumented version
    func.block = Box::new(syn::parse2(new_block).unwrap());
    TokenStream::from(quote! { #func })
}
