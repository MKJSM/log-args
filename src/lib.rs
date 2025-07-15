extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Pat, Lit, Meta};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;

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
                },
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
                },
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
                                    let value = value_str.trim_start_matches('"').trim_end_matches('"');
                                    Some((key, Lit::Str(syn::LitStr::new(value, proc_macro2::Span::call_site()))))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    args.push(AttributeArg::Custom(pairs));
                },
                _ => {
                    // Unknown attribute - report it for better error messages
                    if let Some(ident) = meta.path().get_ident() {
                        args.push(AttributeArg::Unknown(ident.to_string()));
                    }
                },
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
    let mut field_names = Vec::new();
    let mut custom_kvs = Vec::new();
    let mut unknown_attrs = Vec::new();
    
    for arg in attr_args.args {
        match arg {
            AttributeArg::Span => {
                has_span = true;
            },
            AttributeArg::Fields(fields) => {
                field_names.extend(fields);
            },
            AttributeArg::Custom(pairs) => {
                custom_kvs.extend(pairs);
            },
            AttributeArg::Unknown(name) => {
                unknown_attrs.push(name);
            }
        }
    }
    
    // Return error for unknown attributes
    if !unknown_attrs.is_empty() {
        let error_msg = format!("unexpected end of input, Unknown attribute `{}`, expected `fields`, `custom` or `span`", unknown_attrs.join(", "));
        return syn::Error::new(proc_macro2::Span::call_site(), error_msg)
            .to_compile_error()
            .into();
    }

    // Capitalize first letter of function name for better log readability
    let mut chars = fn_name.chars();
    let fn_name_display = match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    };
    
    // Create field expressions based on field names or all function arguments
    let log_fields = if !field_names.is_empty() {
        // User specified `fields(...)`, so we only log those.
        let mut field_exprs = Vec::new();
        
        for field_name in field_names {
            // Handle subfields with dot notation (e.g., user.id)
            if field_name.contains('.') {
                let parts: Vec<&str> = field_name.split('.').collect();
                if parts.len() == 2 {
                    let parent = parts[0];
                    let child = parts[1];
                    let display_name = format!("{}_{}" ,parent, child);
                    let parent_ident = syn::Ident::new(parent, proc_macro2::Span::call_site());
                    let child_ident = syn::Ident::new(child, proc_macro2::Span::call_site());
                    let display_ident = syn::Ident::new(&display_name, proc_macro2::Span::call_site());
                    field_exprs.push(quote! { 
                        #display_ident = tracing::field::debug(&#parent_ident.#child_ident)
                    });
                }
            } else {
                // Simple field
                let ident = syn::Ident::new(&field_name, proc_macro2::Span::call_site());
                field_exprs.push(quote! { 
                    #ident = tracing::field::debug(&#ident)
                });
            }
        }
        field_exprs
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
    let custom_field_tokens = custom_kvs.iter().map(|(key, val)| {
        let key_ident = syn::Ident::new(key, proc_macro2::Span::call_site());
        quote! { #key_ident = #val }
    }).collect::<Vec<_>>();

    // Generate the log field expressions for the span
    let span_fields = quote! {
        function = #fn_name,
        message = #fn_name_display,
    };
    
    // Add function name as a message field to ensure it appears in logs
    // The behavior is the same for both span and non-span cases, but we keep the conditional
    // for future extensibility
    let new_block = if has_span {
        quote! {
            {
                // Create a span with minimal fields
                let span = tracing::span!(
                    tracing::Level::INFO, 
                    #fn_name,
                    #span_fields
                );
                
                // Enter the span and log all arguments as fields
                let _guard = span.enter();
                
                // Log function arguments as fields
                tracing::debug!(
                    #(#log_fields,)*
                    #(#custom_field_tokens,)*
                );
                
                #orig_block
            }
        }
    } else {
        quote! {
            {
                // Create a span with minimal fields
                let span = tracing::span!(
                    tracing::Level::INFO, 
                    #fn_name,
                    #span_fields
                );
                
                // Enter the span and log all arguments as fields
                let _guard = span.enter();
                
                // Log function arguments as fields
                tracing::debug!(
                    #(#log_fields,)*
                    #(#custom_field_tokens,)*
                );
                
                #orig_block
            }
        }
    };

    // Replace the function body with our instrumented version
    func.block = Box::new(syn::parse2(new_block).unwrap());
    TokenStream::from(quote! { #func })
}
