
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{parse_macro_input, ReturnType};


/// Mark a function as an integration test for the server.
///
/// `inventory` and `error-stack` crates are required
/// dependencies when using this macro.
///
/// Example:
///
/// ```
/// use std::future::Future;
/// use test_mode_macro::server_test;
///
/// /// This must be defined in the root of the crate.
/// #[derive(thiserror::Error, Debug)]
/// pub enum TestError {
///    #[error("Error")]
///    Err,
/// }
///
/// /// This must be defined in the root of the crate.
/// pub struct TestContext;
///
/// /// This must be defined in the root of the crate.
/// pub struct TestFunction {
///     name: &'static str,
///     module_path: &'static str,
///     function: fn(TestContext) -> Box<dyn Future<Output = error_stack::Result<(), TestError>>>,
/// }
///
/// impl TestFunction {
///     pub const fn new(
///         name: &'static str,
///         module_path: &'static str,
///         function: fn(TestContext) -> Box<dyn Future<Output = error_stack::Result<(), TestError>>>,
///     ) -> Self {
///         Self {
///             name,
///             module_path,
///             function,
///         }
///     }
/// }
///
/// inventory::collect!(TestFunction);
///
/// #[server_test]
/// async fn hello_register(context: TestContext) -> error_stack::Result<(), TestError> {
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

    if test_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(test_fn.sig.fn_token, "test function must be async")
            .to_compile_error()
            .into();
    }

    let expanded = quote::quote! {
        #test_fn

        fn #hidden_fn_name(
            test_context: crate::TestContext,
        ) -> Box<dyn std::future::Future<Output = error_stack::Result<(), crate::TestError>>> {
            Box::new(#test_fn_name(test_context))
        }

        inventory::submit! {
            crate::TestFunction::new(
                stringify!(#test_fn_name),
                module_path!(),
                #hidden_fn_name,
            )
        }
    };

    TokenStream::from(expanded)
}
