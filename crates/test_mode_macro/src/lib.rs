
use proc_macro::TokenStream;
use syn::parse_macro_input;


/// Mark a function as an integration test for the server.
///
/// `inventory` crate is required dependency when using this macro.
///
/// Example:
///
/// ```
/// use test_mode_macro::server_test;
///
/// /// This must be defined in the root of the crate.
/// pub struct TestFunction {
///     name: &'static str,
///     module_path: &'static str,
///     function: fn(),
/// }
///
/// impl TestFunction {
///     pub const fn new(name: &'static str, module_path: &'static str, function: fn()) -> Self {
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
/// fn test_function() {}
///
/// fn main() {
///     let test_functions = inventory::iter::<TestFunction>;
/// }
/// ```
///
#[proc_macro_attribute]
pub fn server_test(_attr: TokenStream, input: TokenStream) -> TokenStream {

    let test_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
    let test_fn_name = &test_fn.sig.ident;

    let expanded = quote::quote! {
        #test_fn
        inventory::submit! {
            crate::TestFunction::new(
                stringify!(#test_fn_name),
                module_path!(),
                #test_fn_name,
            )
        }
    };

    TokenStream::from(expanded)
}
