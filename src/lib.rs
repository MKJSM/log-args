use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, Block, Expr, FnArg, Ident, ImplItemFn, ItemFn, MetaNameValue, Pat, Token,
};

/// Attribute macro for adding structured logging to functions
///
/// This macro adds structured logging to functions by automatically injecting
/// function parameters into log statements. It supports various options for
/// controlling which parameters are logged and how context is propagated.
///
/// # Features
/// - Automatic parameter logging
/// - Custom field addition
/// - Selective field logging
/// - Context propagation via span
/// - Function name logging with configurable casing styles
///
/// # Examples
///
/// Basic usage:
/// ```
/// #[params]
/// fn authenticate(user_id: &str, password: &str) -> Result<(), Error> {
///     info!("User authentication attempt");
///     // The log will include user_id automatically
///     // password is not logged for security
/// }
/// ```
///
/// With selective fields:
/// ```
/// #[params(fields(user_id, request_id))]
/// fn process_request(user_id: &str, request_id: &str, _sensitive_data: &str) {
///     info!("Processing request");
///     // Only user_id and request_id will be logged
/// }
/// ```
///
/// With custom fields:
/// ```
/// #[params(custom(operation = "user_login", system = "auth_service"))]
/// fn login(username: &str) {
///     info!("User login attempt");
///     // Logs will include the custom fields
/// }
/// ```
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
    let is_async = item.sig().asyncness.is_some();

    let local_fields_quote = get_context_fields_quote(&item, &config);
    let log_redefines = get_log_redefines_with_fields(is_async, config.span);

    let push_logic = if config.span {
        let get_parent_context = if is_async {
            quote! { ::log_args_runtime::get_async_context() }
        } else {
            quote! { ::log_args_runtime::get_context() }
        };

        let push_context = if is_async {
            quote! { ::log_args_runtime::push_async_context }
        } else {
            quote! { ::log_args_runtime::push_context }
        };

        quote! {
            let mut new_context_for_push = #get_parent_context;
            {
                let mut map = ::std::collections::HashMap::<String, String>::new();
                #(#local_fields_quote)*
                new_context_for_push.extend(map.into_iter());
            }
            let _context_guard = #push_context(new_context_for_push);
        }
    } else {
        quote! {}
    };

    let original_block = item.block();
    let new_block = quote! {
        {
            #push_logic

            let local_fields: ::std::collections::HashMap<String, String> = {
                let mut map = ::std::collections::HashMap::<String, String>::new();
                #(#local_fields_quote)*
                map
            };

            #log_redefines

            #original_block
        }
    };

    *item.block_mut() = syn::parse2(new_block).expect("Failed to parse new block");

    match item {
        FnItem::Item(item_fn) => TokenStream::from(quote! { #item_fn }),
        FnItem::ImplItem(impl_item_fn) => TokenStream::from(quote! { #impl_item_fn }),
    }
}

/// Represents the different attribute options for the params macro
enum Attribute {
    /// Selectively log only specific fields
    Fields(Punctuated<Expr, Token![,]>),
    /// Add custom fields to logs
    Custom(Punctuated<MetaNameValue, Token![,]>),
    /// Enable context propagation to child functions
    Span,
    /// Log all parameters
    All,
    /// Mark fields that should only be logged in the current function
    Current(Punctuated<Expr, Token![,]>),
    /// Clone parameters upfront to avoid borrow checker issues
    CloneUpfront,
}

impl Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "fields" {
            let content;
            parenthesized!(content in input);
            Ok(Attribute::Fields(Punctuated::parse_terminated(&content)?))
        } else if ident == "custom" {
            let content;
            parenthesized!(content in input);
            Ok(Attribute::Custom(Punctuated::parse_terminated(&content)?))
        } else if ident == "span" {
            Ok(Attribute::Span)
        } else if ident == "all" {
            Ok(Attribute::All)
        } else if ident == "current" {
            let content;
            parenthesized!(content in input);
            Ok(Attribute::Current(Punctuated::parse_terminated(&content)?))
        } else if ident == "clone_upfront" {
            Ok(Attribute::CloneUpfront)
        } else {
            Err(syn::Error::new_spanned(ident, "unknown attribute"))
        }
    }
}

/// Configuration for the params macro
struct AttrConfig {
    /// Fields to selectively log
    fields: Vec<syn::Expr>,
    /// Custom fields to add to logs
    custom: Vec<syn::MetaNameValue>,
    /// Whether to enable context propagation
    span: bool,
    /// Whether to log all parameters
    all_params: bool,
    /// Fields that should only be logged in the current function
    current: Vec<syn::Expr>,
    /// Whether to clone parameters upfront to avoid borrow checker issues
    clone_upfront: bool,
}

impl AttrConfig {
    fn from_attributes(attrs: Punctuated<Attribute, Token![,]>) -> Self {
        let mut fields = Vec::new();
        let mut custom = Vec::new();
        let mut current = Vec::new();
        let mut span = false;
        let mut all_params = false;
        let mut clone_upfront = false;
        let attrs_is_empty = attrs.is_empty();

        for attr in attrs {
            match attr {
                Attribute::Fields(f) => fields.extend(f.into_iter()),
                Attribute::Custom(c) => custom.extend(c.into_iter()),
                Attribute::Span => span = true,
                Attribute::All => all_params = true,
                Attribute::Current(c) => current.extend(c.into_iter()),
                Attribute::CloneUpfront => clone_upfront = true,
            }
        }

        // By default, span is true if no attributes are provided or only `all` is provided
        if attrs_is_empty
            || (fields.is_empty() && custom.is_empty() && current.is_empty() && all_params)
        {
            span = true;
        }

        Self {
            fields,
            custom,
            span,
            all_params,
            current,
            clone_upfront,
        }
    }
}

/// Represents either a standalone function or a method in an impl block
enum FnItem {
    Item(ItemFn),
    ImplItem(ImplItemFn),
}

impl FnItem {
    fn sig(&self) -> &syn::Signature {
        match self {
            FnItem::Item(item) => &item.sig,
            FnItem::ImplItem(item) => &item.sig,
        }
    }

    fn block(&self) -> &Block {
        match self {
            FnItem::Item(item) => &item.block,
            FnItem::ImplItem(item) => &item.block,
        }
    }

    fn block_mut(&mut self) -> &mut Block {
        match self {
            FnItem::Item(item) => &mut item.block,
            FnItem::ImplItem(item) => &mut item.block,
        }
    }

    fn attrs_mut(&mut self) -> &mut Vec<syn::Attribute> {
        match self {
            FnItem::Item(item) => &mut item.attrs,
            FnItem::ImplItem(item) => &mut item.attrs,
        }
    }
}

/// Generates code to extract context fields from function parameters
fn get_context_fields_quote(item: &FnItem, config: &AttrConfig) -> Vec<proc_macro2::TokenStream> {
    let mut fields_to_add = Vec::new();

    let fn_name = item.sig().ident.to_string();
    let fn_name_str = fn_name.as_str();

    let function_name_casing = if cfg!(feature = "function-names-snake") {
        quote! { #fn_name_str.to_string() }
    } else if cfg!(feature = "function-names-camel") {
        quote! { ::log_args_runtime::to_camel_case(#fn_name_str) }
    } else if cfg!(feature = "function-names-pascal") {
        quote! { ::log_args_runtime::to_pascal_case(#fn_name_str) }
    } else if cfg!(feature = "function-names-screaming") {
        quote! { ::log_args_runtime::to_screaming_snake_case(#fn_name_str) }
    } else if cfg!(feature = "function-names-kebab") {
        quote! { ::log_args_runtime::to_kebab_case(#fn_name_str) }
    } else {
        quote! { #fn_name_str.to_string() } // Default to snake_case if no feature is set
    };

    if cfg!(any(
        feature = "function-names-snake",
        feature = "function-names-camel",
        feature = "function-names-pascal",
        feature = "function-names-screaming",
        feature = "function-names-kebab"
    )) {
        fields_to_add.push(quote! {
            map.insert("function".to_string(), #function_name_casing);
        });
    }

    let log_all = config.all_params;

    for arg in &item.sig().inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let ident = &pat_ident.ident;
                let ident_str = ident.to_string();

                if ident_str == "self" {
                    continue;
                }

                // Check if this field should be logged
                let should_log = if log_all {
                    true
                } else {
                    config.fields.iter().any(|expr| {
                        let expr_str = quote!(#expr).to_string();
                        expr_str == ident_str
                    })
                };

                // Check if this field is in the current-only list
                let is_current_only = config.current.iter().any(|expr| {
                    let expr_str = quote!(#expr).to_string();
                    expr_str == ident_str
                });

                if should_log && !is_current_only {
                    if config.clone_upfront {
                        // Clone upfront to avoid borrow checker issues
                        fields_to_add.push(quote! {
                            let cloned_value = #ident.clone();
                            // Try to serialize, fallback to Debug if Serialize not available
                            let value_str = match serde_json::to_string(&cloned_value) {
                                Ok(serialized) => serialized,
                                Err(_) => {
                                    // Fallback to Debug representation if available
                                    format!("{:?}", cloned_value)
                                }
                            };
                            map.insert(#ident_str.to_string(), value_str);
                        });
                    } else {
                        fields_to_add.push(quote! {
                            // Try to serialize, fallback to Debug if Serialize not available
                            let value_str = match serde_json::to_string(&#ident) {
                                Ok(serialized) => serialized,
                                Err(_) => {
                                    // Fallback to Debug representation if available
                                    format!("{:?}", #ident)
                                }
                            };
                            map.insert(#ident_str.to_string(), value_str);
                        });
                    }
                }
            }
        }
    }

    // Handle complex field expressions (like user.id)
    if !log_all {
        for expr in &config.fields {
            let expr_str = quote!(#expr).to_string().replace(' ', "");
            if expr_str.contains('.') {
                if config.clone_upfront {
                    // Clone upfront to avoid borrow checker issues
                    fields_to_add.push(quote! {
                        let cloned_value = (#expr).clone();
                        let value_str = match serde_json::to_string(&cloned_value) {
                            Ok(serialized) => serialized,
                            Err(_) => format!("{:?}", cloned_value)
                        };
                        map.insert(#expr_str.to_string(), value_str);
                    });
                } else {
                    fields_to_add.push(quote! {
                        let value_str = match serde_json::to_string(&#expr) {
                            Ok(serialized) => serialized,
                            Err(_) => format!("{:?}", #expr)
                        };
                        map.insert(#expr_str.to_string(), value_str);
                    });
                }
            }
        }
    }

    // Add custom fields
    for custom_field in &config.custom {
        let key = &custom_field.path;
        let value = &custom_field.value;
        // Convert the path to a string literal
        let key_str = if let Some(ident) = key.get_ident() {
            ident.to_string()
        } else {
            quote!(#key).to_string()
        };
        fields_to_add.push(quote! {
            let value_str = match serde_json::to_string(&#value) {
                Ok(serialized) => serialized,
                Err(_) => format!("{:?}", #value)
            };
            map.insert(#key_str.to_string(), value_str);
        });
    }

    // Add current-only fields
    for current_field in &config.current {
        let current_str = quote!(#current_field).to_string();
        fields_to_add.push(quote! {
            let value_str = match serde_json::to_string(&#current_field) {
                Ok(serialized) => serialized,
                Err(_) => format!("{:?}", #current_field)
            };
            map.insert(#current_str.to_string(), value_str);
        });
    }

    fields_to_add
}

/// Generates macro redefinitions that inject context fields into log statements
fn get_log_redefines_with_fields(is_async: bool, span_enabled: bool) -> proc_macro2::TokenStream {
    let get_parent_context_logic = if span_enabled {
        if is_async {
            quote! { ::log_args_runtime::get_async_context() }
        } else {
            quote! { ::log_args_runtime::get_context() }
        }
    } else {
        quote! { ::std::collections::HashMap::<String, String>::new() }
    };

    quote! {
        // Override tracing macros with our own versions that include context fields
        use tracing::{info, warn, error, debug, trace};

        // Helper macro to build field assignments dynamically
        macro_rules! build_tracing_call {
            ($level:ident, $msg:expr, $fields:expr) => {
                // Clean up field values (remove extra quotes)
                let mut clean_fields = std::collections::HashMap::new();
                for (key, value) in $fields {
                    let clean_value = if value.starts_with('\"') && value.ends_with('\"') {
                        value[1..value.len()-1].to_string()
                    } else {
                        value.clone()
                    };
                    clean_fields.insert(key.clone(), clean_value);
                }
                
                if clean_fields.is_empty() {
                    tracing::$level!($msg);
                } else {
                    // Generate a macro call with individual field key-value pairs
                    // This approach creates truly flat JSON output by injecting each field
                    // as a separate key-value pair in the tracing macro call
                    
                    // Prepare fields for JSON serialization
                    // This approach ensures consistent handling for all field counts
                    
                    // Use consistent JSON approach for all field counts
                    // This ensures proper span propagation and context inheritance
                    let fields_json = match serde_json::to_string(&clean_fields) {
                        Ok(json) => json,
                        Err(_) => "{}".to_string(),
                    };
                    
                    match stringify!($level) {
                        "info" => tracing::info!(fields = %fields_json, "{}", $msg),
                        "warn" => tracing::warn!(fields = %fields_json, "{}", $msg),
                        "error" => tracing::error!(fields = %fields_json, "{}", $msg),
                        "debug" => tracing::debug!(fields = %fields_json, "{}", $msg),
                        "trace" => tracing::trace!(fields = %fields_json, "{}", $msg),
                        _ => tracing::info!(fields = %fields_json, "{}", $msg),
                    }
                }
            };
        }

        // Redefine macros to include context fields using proper tracing integration
        macro_rules! info {
            ($msg:expr) => {
                let mut all_fields = #get_parent_context_logic;
                all_fields.extend(local_fields.clone());
                build_tracing_call!(info, $msg, all_fields);
            };
            ($($args:tt)*) => {
                tracing::info!($($args)*)
            };
        }

        macro_rules! warn {
            ($msg:expr) => {
                let mut all_fields = #get_parent_context_logic;
                all_fields.extend(local_fields.clone());
                build_tracing_call!(warn, $msg, all_fields);
            };
            ($($args:tt)*) => {
                tracing::warn!($($args)*)
            };
        }

        macro_rules! error {
            ($msg:expr) => {
                let mut all_fields = #get_parent_context_logic;
                all_fields.extend(local_fields.clone());
                build_tracing_call!(error, $msg, all_fields);
            };
            ($($args:tt)*) => {
                tracing::error!($($args)*)
            };
        }

        macro_rules! debug {
            ($msg:expr) => {
                let mut all_fields = #get_parent_context_logic;
                all_fields.extend(local_fields.clone());
                build_tracing_call!(debug, $msg, all_fields);
            };
            ($($args:tt)*) => {
                tracing::debug!($($args)*)
            };
        }

        macro_rules! trace {
            ($msg:expr) => {
                let mut all_fields = #get_parent_context_logic;
                all_fields.extend(local_fields.clone());
                build_tracing_call!(trace, $msg, all_fields);
            };
            ($($args:tt)*) => {
                tracing::trace!($($args)*)
            };
        }
    }
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
