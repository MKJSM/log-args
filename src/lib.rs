//! # log_args
//!
//! A production-ready procedural macro library for automatic function argument logging
//! with structured tracing support, context propagation, and security-conscious design.
//!
//! ## Features
//!
//! - **Automatic Parameter Logging**: Log all or selected function parameters
//! - **Selective Field Logging**: Choose specific fields to log for security and performance
//! - **Custom Static Fields**: Add service metadata and static context to logs
//! - **Span Context Propagation**: Automatically propagate context to child functions
//! - **Function Name Logging**: Include function names with configurable casing styles
//! - **Async Support**: Full compatibility with async/await and tokio
//! - **Method Support**: Works with impl blocks and methods
//! - **Security-First**: Designed to exclude sensitive data by default
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! log_args = "0.1"
//! tracing = "0.1"
//! ```
//!
//! Basic usage:
//! ```rust
//! use log_args::params;
//! use tracing::info;
//!
//! #[params]
//! fn authenticate_user(username: String, password: String) {
//!     info!("User authentication attempt");
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! **Always use selective logging in production:**
//! ```rust
//! // Good - Only logs safe fields
//! #[params(fields(user.id, operation_type))]
//! fn secure_operation(user: User, password: String, operation_type: String) {
//!     info!("Operation started");
//! }
//!
//! // Bad - Logs everything including sensitive data
//! #[params]
//! fn insecure_operation(user: User, password: String) {
//!     // This would log the password!
//! }
//! ```
//!
//! See the [USAGE.md](https://github.com/MKJSM/log-args/blob/main/USAGE.md) for comprehensive documentation.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Expr, FnArg, Ident, MetaNameValue, Pat, Token};

/// A procedural macro for automatic function argument logging with structured tracing.
///
/// **By default, `#[params]` enables span-based context propagation and function name logging.**
/// This provides comprehensive observability with minimal configuration.
///
/// This macro provides multiple modes of operation for different use cases:
/// - Basic parameter logging with span propagation (default)
/// - Selective field logging for security
/// - Custom static field injection
/// - Function name logging with configurable casing
/// - All parameters logging with `all` attribute
///
/// ## Basic Usage (Default: Span + Function Names)
///
/// ```rust
/// use log_args::params;
/// use tracing::info;
///
/// #[params]
/// fn process_user(user_id: u64, username: String) {
///     info!("Processing user data");
///     // Child functions will inherit context automatically
///     validate_user(user_id);
/// }
///
/// #[params]
/// fn validate_user(id: u64) {
///     info!("Validating user"); // Inherits parent context
/// }
/// ```
///
/// **JSON Output (with function names enabled via Cargo feature):**
/// ```json
/// {
///   "level": "INFO",
///   "fields": {
///     "message": "Processing user data",
///     "function": "ProcessUser",
///     "user_id": 12345,
///     "username": "alice"
///   }
/// }
/// ```
///
/// ## Selective Field Logging (Recommended for Production)
///
/// For security and performance, specify exactly which fields to log:
///
/// ```rust
/// #[params(fields(user.id, operation_type))]
/// fn secure_operation(
///     user: User,
///     password: String,      // Not logged - sensitive
///     operation_type: String, // Logged - safe
/// ) {
///     info!("Secure operation started");
/// }
/// ```
///
/// ## Custom Static Fields
///
/// Add service metadata and static context:
///
/// ```rust
/// #[params(
///     fields(user_id),
///     custom(
///         service = "user-management",
///         version = "2.1.0",
///         environment = "production"
///     )
/// )]
/// fn service_operation(user_id: u64, sensitive_data: String) {
///     info!("Service operation");
/// }
/// ```
///
/// ## All Parameters Logging
///
/// Use the `all` attribute to explicitly log all function parameters:
///
/// ```rust
/// #[params(all)]
/// fn debug_function(user_id: u64, data: String, config: Config) {
///     info!("Debug information");
/// }
/// ```
///
/// This is useful for debugging or when you want to ensure all parameters are logged
/// regardless of other attributes.
///
/// ## Span Context Propagation (Enabled by Default)
///
/// **Note: Span propagation is now enabled by default with `#[params]`.**
/// Context automatically propagates to child functions:
///
/// ```rust
/// use log_args_runtime::{info as ctx_info};
///
/// #[params(fields(user.id, transaction.amount))]
/// fn process_payment(user: User, transaction: Transaction, card_data: CardData) {
///     info!("Starting payment processing");
///     
///     validate_payment();  // Inherits context automatically
///     charge_card();       // Inherits context automatically
/// }
///
/// #[params]
/// fn validate_payment() {
///     info!("Validating payment");  // Includes parent context
/// }
/// ```
///
/// ## Function Name Logging
///
/// Enable function name logging with Cargo features:
///
/// ```toml
/// [dependencies]
/// log_args = { version = "0.1", features = ["function-names-pascal"] }
/// ```
///
/// Available casing styles:
/// - `function-names-snake` → `process_payment`
/// - `function-names-camel` → `processPayment`
/// - `function-names-pascal` → `ProcessPayment` (recommended)
/// - `function-names-screaming` → `PROCESS_PAYMENT`
/// - `function-names-kebab` → `process-payment`
///
/// ## Async Support
///
/// Works seamlessly with async functions:
///
/// ```rust
/// #[params(span, fields(user_id, operation_type))]
/// async fn async_operation(user_id: u64, operation_type: String, secret: String) {
///     info!("Starting async operation");
///     
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     
///     info!("Async operation completed");
/// }
/// ```
///
/// ## Method Support
///
/// Works with methods in impl blocks:
///
/// ```rust
/// impl UserService {
///     #[params(span, fields(user.id, self.config.timeout))]
///     fn process_user(&self, user: User, sensitive_token: String) {
///         info!("Processing user in service");
///     }
/// }
/// ```
///
/// ## Security Considerations
///
/// **⚠️ Important:** Always use selective logging in production to avoid logging sensitive data:
///
/// - Passwords, tokens, API keys
/// - Personal Identifiable Information (PII)
/// - Credit card numbers, financial data
/// - Internal system keys and secrets
///
/// ## Error Handling
///
/// The macro works with Result types and error handling patterns:
///
/// ```rust
/// #[params(fields(operation_id, retry_count))]
/// fn fallible_operation(
///     operation_id: String,
///     retry_count: u32,
///     secret_key: String,  // Not logged
/// ) -> Result<String, ProcessingError> {
///     info!("Starting fallible operation");
///     
///     // Operation logic that might fail
///     Ok("success".to_string())
/// }
/// ```
///
/// ## Performance Notes
///
/// - Selective logging (`fields(...)`) is more efficient than logging all parameters
/// - Complex field expressions are evaluated at runtime - use judiciously in hot paths
/// - Span creation has overhead - use for important operations that benefit from context
///
/// For comprehensive documentation and examples, see:
/// - [USAGE.md](https://github.com/MKJSM/log-args/blob/main/USAGE.md)
/// - [Examples](https://github.com/MKJSM/log-args/tree/main/examples)
/// - [Integration Tests](https://github.com/MKJSM/log-args/tree/main/tests)
///
/// fn child_function() {
///     info!("Child task");
/// }
///
#[proc_macro_attribute]
pub fn params(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = if let Ok(item_fn) = syn::parse::<syn::ItemFn>(input.clone()) {
        FnItem::Item(item_fn)
    } else if let Ok(impl_item_fn) = syn::parse::<syn::ImplItemFn>(input.clone()) {
        FnItem::ImplItem(impl_item_fn)
    } else {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(input),
            "The #[params] attribute can only be applied to functions or methods.",
        )
        .to_compile_error()
        .into();
    };

    let allow_unused_macros_attr: syn::Attribute = syn::parse_quote! { #[allow(unused_macros)] };
    item.attrs_mut().push(allow_unused_macros_attr);

    let attrs = match Punctuated::<Attribute, Token![,]>::parse_terminated.parse(args) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let config = AttrConfig::from_attributes(attrs);
    let context_fields = get_context_fields_quote(&item, &config);

    if config.span {
        // Generate context map for span propagation
        let context_map = get_context_map_for_span(&item, &config);

        if item.sig().asyncness.is_some() {
            let log_redefines = get_log_redefines_with_fields(&context_fields, true);
            let original_block = item.block();
            let new_block = quote! {
                {
                    let _context_guard = ::log_args_runtime::push_async_context(#context_map);
                    #log_redefines
                    #original_block
                }
            };
            *item.block_mut() = syn::parse2(new_block).expect("Failed to parse new async block");
        } else {
            let log_redefines = get_log_redefines_with_fields(&context_fields, false);
            let original_block = item.block();
            let new_block = quote! {
                {
                    let _context_guard = ::log_args_runtime::push_context(#context_map);
                    #log_redefines
                    #original_block
                }
            };
            *item.block_mut() = syn::parse2(new_block).expect("Failed to parse new sync block");
        }
    } else {
        // No span, use direct field injection
        if item.sig().asyncness.is_some() {
            let log_redefines = get_log_redefines_with_fields(&context_fields, true);
            let original_block = item.block();
            let new_block = quote! {
                {
                    #log_redefines
                    #original_block
                }
            };
            *item.block_mut() = syn::parse2(new_block).expect("Failed to parse new async block");
        } else {
            let log_redefines = get_log_redefines_with_fields(&context_fields, false);
            let original_block = item.block();
            let new_block = quote! {
                {
                    #log_redefines
                    #original_block
                }
            };
            *item.block_mut() = syn::parse2(new_block).expect("Failed to parse new sync block");
        }
    }

    TokenStream::from(quote! { #item })
}

enum Attribute {
    Fields(Punctuated<Expr, Token![,]>),
    Custom(Punctuated<MetaNameValue, Token![,]>),
    Current(Punctuated<Expr, Token![,]>),
    CloneUpfront,
    Span,
    All,
    AutoCapture,  // New attribute for automatic closure context capture
}

impl Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "fields" {
            let content;
            parenthesized!(content in input);
            let fields = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
            Ok(Attribute::Fields(fields))
        } else if ident == "custom" {
            let content;
            parenthesized!(content in input);
            let custom = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(&content)?;
            Ok(Attribute::Custom(custom))
        } else if ident == "current" {
            let content;
            parenthesized!(content in input);
            let current = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
            Ok(Attribute::Current(current))
        } else if ident == "clone_upfront" {
            Ok(Attribute::CloneUpfront)
        } else if ident == "span" {
            Ok(Attribute::Span)
        } else if ident == "all" {
            Ok(Attribute::All)
        } else if ident == "auto_capture" {
            Ok(Attribute::AutoCapture)
        } else {
            Err(syn::Error::new_spanned(ident, "unknown attribute"))
        }
    }
}

struct AttrConfig {
    fields: Vec<syn::Expr>,
    custom: Vec<syn::MetaNameValue>,
    current: Vec<syn::Expr>,
    clone_upfront: bool,
    span: bool,
    all_params: bool,
    auto_capture: bool,  // New field for automatic closure context capture
}

impl Default for AttrConfig {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            custom: Vec::new(),
            current: Vec::new(),
            clone_upfront: true, // Default to true for safety
            span: true,          // Default to true for context propagation
            all_params: false,
            auto_capture: false, // Default to false for auto_capture
        }
    }
}

impl AttrConfig {
    fn from_attributes(attrs: Punctuated<Attribute, Token![,]>) -> Self {
        let mut config = AttrConfig::default();
        for attr in attrs {
            match attr {
                Attribute::Fields(fields) => config.fields.extend(fields),
                Attribute::Custom(custom) => config.custom.extend(custom),
                Attribute::Current(current) => config.current.extend(current),
                Attribute::CloneUpfront => config.clone_upfront = true,
                Attribute::Span => {
                    config.span = true;
                    config.clone_upfront = true; // Span implies clone_upfront for safety
                }
                Attribute::All => {
                    config.all_params = true;
                }
                Attribute::AutoCapture => {
                    config.auto_capture = true;
                }
            }
        }
        config
    }
}

fn get_context_fields_quote(item: &FnItem, config: &AttrConfig) -> Vec<proc_macro2::TokenStream> {
    let mut field_assignments = vec![];

    // Determine what to log based on configuration
    let _has_selective_attributes =
        !config.fields.is_empty() || !config.custom.is_empty() || !config.current.is_empty();

    // For span propagation, automatically inherit parent context fields
    // This ensures automatic context propagation without manual attributes
    if config.span && config.fields.is_empty() && config.custom.is_empty() && config.current.is_empty() && !config.all_params {
        // When only span is enabled (default behavior), inherit all parent context fields
        // This uses the runtime macro to dynamically include inherited fields
        field_assignments.push(quote! {
            context = ::log_args_runtime::get_inherited_context_string()
        });
    }

    if config.all_params {
        // Log all parameters only when 'all' is explicitly specified
        let all_args = get_all_args(item);
        for ident in all_args {
            let ident_str = ident.to_string();
            // When span is enabled, use span context lookup for post-move safety
            if config.span {
                field_assignments.push(quote! { 
                    #ident = ::log_args_runtime::get_context_value(&#ident_str).unwrap_or_else(|| "<missing>".to_string())
                });
            } else {
                field_assignments.push(quote! { #ident = ?#ident });
            }
        }
    }

    if !config.fields.is_empty() {
        // Log only specified fields
        for field_expr in &config.fields {
            // Convert complex expressions to string field names
            let field_name = quote! { #field_expr }.to_string();
            let field_key = field_name.replace(' ', "");
            
            // If clone_upfront is enabled and expression contains self.field, handle it specially
            if config.clone_upfront {
                let expr_str = quote!(#field_expr).to_string();
                if expr_str.contains("self.") {
                    // When span is enabled, use span context lookup for post-move safety
                    if config.span {
                        field_assignments.push(quote! { 
                            #field_name = ::log_args_runtime::get_context_value(&#field_key).unwrap_or_else(|| "<missing>".to_string())
                        });
                    } else {
                        // No span, use cloned variable approach (similar to custom fields)
                        let mut modified_expr_str = expr_str.clone();
                        let mut start = 0;
                        while let Some(pos) = modified_expr_str[start..].find("self.") {
                            let field_start = start + pos + 5; // Skip "self."
                            let remaining = &modified_expr_str[field_start..];
                            
                            // Find the end of the field name
                            let field_end = remaining
                                .find(|c: char| !c.is_alphanumeric() && c != '_')
                                .unwrap_or(remaining.len());
                            
                            let field_name_part = &remaining[..field_end];
                            let replacement = format!("__{}_for_macro", field_name_part);
                            
                            // Replace self.field_name with __field_name_for_macro
                            let old_expr = format!("self.{}", field_name_part);
                            modified_expr_str = modified_expr_str.replace(&old_expr, &replacement);
                            
                            start = field_start + field_end;
                        }
                        
                        // Parse the modified string back to a token stream
                        let modified_expr: proc_macro2::TokenStream = modified_expr_str.parse().unwrap_or_else(|_| quote!(#field_expr));
                        field_assignments.push(quote! { #field_name = ?#modified_expr });
                    }
                } else {
                    field_assignments.push(quote! { #field_name = ?#field_expr });
                }
            } else {
                field_assignments.push(quote! { #field_name = ?#field_expr });
            }
        }
    }
    // Default behavior: Only enable span propagation and function name logging
    // No automatic parameter logging unless explicitly requested
    // If only custom/current are specified (no fields), we don't log any parameters

    // Add custom fields (always included)
    for nv in &config.custom {
        let key = &nv.path;
        let value = &nv.value;
        
        // Add to logging fields
        field_assignments.push(quote! {
            #key = #value
        });
    }

    // Add current fields (only logged in current function, not propagated)
    for current_field in &config.current {
        let field_name = quote! { #current_field }.to_string();
        let field_key = field_name.replace(' ', "");
        
        // If clone_upfront is enabled and expression contains self.field, handle it specially
        if config.clone_upfront {
            let expr_str = quote!(#current_field).to_string();
            if expr_str.contains("self.") {
                // When span is enabled, use span context lookup for post-move safety
                if config.span {
                    field_assignments.push(quote! { 
                        #field_name = ::log_args_runtime::get_context_value(&#field_key).unwrap_or_else(|| "<missing>".to_string())
                    });
                } else {
                    // No span, use cloned variable approach (similar to custom fields)
                    let mut modified_expr_str = expr_str.clone();
                    let mut start = 0;
                    while let Some(pos) = modified_expr_str[start..].find("self.") {
                        let field_start = start + pos + 5; // Skip "self."
                        let remaining = &modified_expr_str[field_start..];
                        
                        // Find the end of the field name
                        let field_end = remaining
                            .find(|c: char| !c.is_alphanumeric() && c != '_')
                            .unwrap_or(remaining.len());
                        
                        let field_name_part = &remaining[..field_end];
                        let replacement = format!("__{}_for_macro", field_name_part);
                        
                        // Replace self.field_name with __field_name_for_macro
                        let old_expr = format!("self.{}", field_name_part);
                        modified_expr_str = modified_expr_str.replace(&old_expr, &replacement);
                        
                        start = field_start + field_end;
                    }
                    
                    // Parse the modified string back to a token stream
                    let modified_expr: proc_macro2::TokenStream = modified_expr_str.parse().unwrap_or_else(|_| quote!(#current_field));
                    field_assignments.push(quote! { #field_name = ?#modified_expr });
                }
            } else {
                field_assignments.push(quote! { #field_name = ?#current_field });
            }
        } else {
            field_assignments.push(quote! { #field_name = ?#current_field });
        }
    }

    // Add function name if any function-names feature is enabled
    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    {
        let function_name = get_function_name(item);
        field_assignments.push(quote! { "function" = #function_name });
    }

    field_assignments
}

#[cfg(any(
    feature = "function-names-snake",
    feature = "function-names-camel",
    feature = "function-names-pascal",
    feature = "function-names-screaming",
    feature = "function-names-kebab"
))]
fn get_function_name(item: &FnItem) -> String {
    let function_name = match item {
        FnItem::Item(item_fn) => item_fn.sig.ident.to_string(),
        FnItem::ImplItem(impl_item_fn) => impl_item_fn.sig.ident.to_string(),
    };

    // Apply the appropriate casing based on enabled features
    #[cfg(feature = "function-names-snake")]
    return function_name; // Keep original snake_case

    #[cfg(feature = "function-names-camel")]
    return to_camel_case(&function_name);

    #[cfg(feature = "function-names-pascal")]
    return to_pascal_case(&function_name);

    #[cfg(feature = "function-names-screaming")]
    return to_screaming_snake_case(&function_name);

    #[cfg(feature = "function-names-kebab")]
    return to_kebab_case(&function_name);

    // Default fallback when no function name features are enabled
    #[cfg(not(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    )))]
    to_pascal_case(&function_name)
}

// Convert snake_case to camelCase (first letter lowercase)
#[cfg(feature = "function-names-camel")]
fn to_camel_case(snake_case: &str) -> String {
    let words: Vec<&str> = snake_case.split('_').collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        if !word.is_empty() {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push_str(&first.to_uppercase().collect::<String>());
                result.push_str(&chars.as_str().to_lowercase());
            }
        }
    }
    result
}

// Convert snake_case to PascalCase (first letter uppercase)
#[allow(dead_code)]
#[cfg(any(
    feature = "function-names-pascal",
    not(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))
))]
fn to_pascal_case(snake_case: &str) -> String {
    snake_case
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

// Convert snake_case to SCREAMING_SNAKE_CASE
#[cfg(feature = "function-names-screaming")]
fn to_screaming_snake_case(snake_case: &str) -> String {
    snake_case.to_uppercase()
}

// Convert snake_case to kebab-case
#[cfg(feature = "function-names-kebab")]
fn to_kebab_case(snake_case: &str) -> String {
    snake_case.replace('_', "-")
}

fn get_context_map_for_span(_item: &FnItem, config: &AttrConfig) -> proc_macro2::TokenStream {
    let mut fields_to_log = vec![];

    // Store all field types in span context for dynamic lookup
    // This ensures that span context lookup works for ALL field types
    
    // 1. Add all parameters if requested
    if config.all_params {
        let all_args = get_all_args(_item);
        for ident in all_args {
            let ident_str = ident.to_string();
            fields_to_log.push(quote! {
                new_context.insert(#ident_str.to_string(), format!("{:?}", #ident));
            });
        }
    }
    
    // 2. Add explicitly specified fields
    if !config.fields.is_empty() {
        for field_expr in &config.fields {
            let key_str = quote!(#field_expr).to_string().replace(' ', "");
            fields_to_log.push(quote! {
                new_context.insert(#key_str.to_string(), format!("{:?}", &#field_expr));
            });
        }
    }
    
    // 3. Add custom fields (always included)
    for nv in &config.custom {
        let key = &nv.path;
        let value = &nv.value;
        let key_str = quote!(#key).to_string().replace(' ', "");
        
        // For span context, use the original expression directly
        // This will be evaluated before any moves happen
        fields_to_log.push(quote! {
            new_context.insert(#key_str.to_string(), format!("{}", #value));
        });
        
        // Also store globally for cross-boundary persistence
        fields_to_log.push(quote! {
            ::log_args_runtime::set_global_context(&#key_str, &format!("{}", #value));
        });
    }
    
    // 4. Add current fields (these are also stored in context for consistency)
    for current_field in &config.current {
        let field_name = quote! { #current_field }.to_string();
        let field_key = field_name.replace(' ', "");
        fields_to_log.push(quote! {
            new_context.insert(#field_key.to_string(), format!("{:?}", #current_field));
        });
    }

    // Add function name to context if any function-names feature is enabled (always propagated)
    #[cfg(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    ))]
    {
        let function_name = get_function_name(_item);
        fields_to_log.push(quote! {
            new_context.insert("function".to_string(), #function_name.to_string());
        });
    }

    quote! {
        {
            let mut new_context = ::std::collections::HashMap::new();
            #(#fields_to_log)*
            new_context
        }
    }
}

fn get_all_args(item: &FnItem) -> Vec<Ident> {
    item.sig()
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pt) = arg {
                if let Pat::Ident(pi) = &*pt.pat {
                    if pi.ident != "self" {
                        return Some(pi.ident.clone());
                    }
                }
            }
            None
        })
        .collect()
}

enum FnItem {
    Item(syn::ItemFn),
    ImplItem(syn::ImplItemFn),
}

impl quote::ToTokens for FnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnItem::Item(i) => i.to_tokens(tokens),
            FnItem::ImplItem(i) => i.to_tokens(tokens),
        }
    }
}

impl FnItem {
    fn attrs_mut(&mut self) -> &mut Vec<syn::Attribute> {
        match self {
            FnItem::Item(item_fn) => &mut item_fn.attrs,
            FnItem::ImplItem(impl_item_fn) => &mut impl_item_fn.attrs,
        }
    }

    fn sig(&self) -> &syn::Signature {
        match self {
            FnItem::Item(i) => &i.sig,
            FnItem::ImplItem(i) => &i.sig,
        }
    }

    fn block(&self) -> &syn::Block {
        match self {
            FnItem::Item(i) => &i.block,
            FnItem::ImplItem(i) => &i.block,
        }
    }

    fn block_mut(&mut self) -> &mut syn::Block {
        match self {
            FnItem::Item(i) => &mut i.block,
            FnItem::ImplItem(i) => &mut i.block,
        }
    }
}

fn get_log_redefines_with_fields(
    context_fields: &[proc_macro2::TokenStream],
    _is_async: bool,
) -> proc_macro2::TokenStream {
    // Always redefine macros to include both local fields and inherited context
    // The context inheritance will be handled by including context fields from the runtime
    quote! {
        macro_rules! info {
            ($($t:tt)*) => {
                ::log_args_runtime::log_with_context!(::tracing::info, ::log_args_runtime::get_context(), #(#context_fields,)* $($t)*);
            };
        }
        macro_rules! warn {
            ($($t:tt)*) => {
                ::log_args_runtime::log_with_context!(::tracing::warn, ::log_args_runtime::get_context(), #(#context_fields,)* $($t)*);
            };
        }
        macro_rules! error {
            ($($t:tt)*) => {
                ::log_args_runtime::log_with_context!(::tracing::error, ::log_args_runtime::get_context(), #(#context_fields,)* $($t)*);
            };
        }
        macro_rules! debug {
            ($($t:tt)*) => {
                ::log_args_runtime::log_with_context!(::tracing::debug, ::log_args_runtime::get_context(), #(#context_fields,)* $($t)*);
            };
        }
        macro_rules! trace {
            ($($t:tt)*) => {
                ::log_args_runtime::log_with_context!(::tracing::trace, ::log_args_runtime::get_context(), #(#context_fields,)* $($t)*);
            };
        }
    }
}
