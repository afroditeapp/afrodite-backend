use proc_macro::TokenStream;
use quote::format_ident;
use syn::parse_macro_input;

/// Mark a function as an integration test for the server.
///
/// `inventory` crate is required dependency when using this macro.
///
/// Example:
///
/// ```
/// use std::future::Future;
/// use test_mode_bot::server_test;
///
/// inventory::collect!(TestFunction);
///
/// #[server_test]
/// async fn hello_register(mut context: TestContext) -> TestResult {
///     Ok(())
/// }
///
/// fn main() {
///     let test_functions = inventory::iter::<TestFunction>();
/// }
/// ```
///
#[proc_macro_attribute]
pub fn server_test(_attr: TokenStream, input: TokenStream) -> TokenStream {
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

    let expanded = quote::quote! {
        #test_fn

        mod #hidden_mod_name {
            pub fn #hidden_fn_name(
                test_context: test_mode_tests::TestContext,
            ) -> Box<dyn std::future::Future<Output = test_mode_tests::TestResult> + Send> {
                Box::new(super::#test_fn_name(test_context))
            }
        }

        inventory::submit! {
            test_mode_tests::TestFunction {
                name: stringify!(#test_fn_name),
                module_path: module_path!(),
                function: #hidden_mod_name::#hidden_fn_name,
            }
        }
    };

    TokenStream::from(expanded)
}
