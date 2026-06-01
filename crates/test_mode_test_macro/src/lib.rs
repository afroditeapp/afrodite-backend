use proc_macro::TokenStream;
use quote::format_ident;
use syn::{Ident, parse_macro_input};

/// Parsed arguments for `#[server_test(...)]`.
struct ServerTestArgs {
    /// If set, the name of a function that modifies server config.
    modify_server_config_with: Option<Ident>,
}

impl ServerTestArgs {
    fn from_attr(attr: TokenStream) -> Self {
        if attr.is_empty() {
            return Self {
                modify_server_config_with: None,
            };
        }

        // Parse: modify_server_config_with = "function_name"
        let attr = attr.to_string();
        let input = attr
            .trim()
            .strip_prefix("modify_server_config_with = \"")
            .and_then(|v| v.strip_suffix('"'));

        if let Some(value) = input {
            let value = value.trim();
            let ident: Ident = syn::parse_str(value)
                .expect("Expected a string literal after 'modify_server_config_with = '");
            Self {
                modify_server_config_with: Some(ident),
            }
        } else {
            panic!(
                "Unknown server_test attribute argument: '{attr}'. \
                 Expected nothing or 'modify_server_config_with = \"function_name\"'."
            );
        }
    }
}

/// Mark a function as an integration test for the server.
///
/// `inventory` crate is required dependency when using this macro.
///
/// ```ignore
/// #[server_test]
/// async fn my_test(mut context: TestContext) -> TestResult {
///     Ok(())
/// }
/// ```
///
/// ```ignore
/// fn my_config_modifier(config: ServerConfigEditor) {
///     config.server.api.debug_disable_api_limits = true;
/// }
///
/// #[server_test(modify_server_config_with = "my_config_modifier")]
/// async fn my_test(mut context: TestContext) -> TestResult {
///     Ok(())
/// }
/// ```
///
#[proc_macro_attribute]
pub fn server_test(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = ServerTestArgs::from_attr(attr);
    let test_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
    let test_fn_name = &test_fn.sig.ident;
    let hidden_fn_name = format_ident!("__hidden_{}", test_fn_name);

    // macOS workaround for inventory library
    //
    // There seems to be max limit of 4 functions per module, otherwise
    // TestFunctions for all functions in the module will be missing.
    // To workaround this, we create a hidden module for each test function.
    //
    // Only debug builds need this workaround.
    let hidden_mod_name = format_ident!("__hidden_mod_{}", test_fn_name);

    if test_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(test_fn.sig.fn_token, "test function must be async")
            .to_compile_error()
            .into();
    }

    let modify_config_field = if let Some(modifier_name) = args.modify_server_config_with.as_ref() {
        quote::quote! {
            modify_config: Some(#modifier_name),
        }
    } else {
        quote::quote! {
            modify_config: None,
        }
    };

    let expanded = quote::quote! {
        #test_fn

        mod #hidden_mod_name {
            pub fn #hidden_fn_name(
                test_context: test_mode_test_utils::TestContext,
            ) -> Box<dyn std::future::Future<Output = test_mode_test_utils::TestResult> + Send> {
                Box::new(super::#test_fn_name(test_context))
            }
        }

        inventory::submit! {
            test_mode_test_utils::TestFunction {
                name: stringify!(#test_fn_name),
                module_path: module_path!(),
                function: #hidden_mod_name::#hidden_fn_name,
                #modify_config_field
            }
        }
    };

    TokenStream::from(expanded)
}
