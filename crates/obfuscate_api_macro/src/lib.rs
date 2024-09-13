use base64::Engine;
use sha1::Digest;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::parse_macro_input;

/// Obfuscate utoipa compatible API route path and generate
/// axum compatible path from it.
///
/// Example:
///
/// ```
/// use obfuscate_api_macro::obfuscate_api;
///
/// #[obfuscate_api]
/// pub const PATH_GET_IMAGE: &str = "/media_api/image/{image_id}";
///
/// fn main() {
///     assert!(PATH_GET_IMAGE_AXUM.ends_with("/:image_id"))
/// }
/// ```
///
#[proc_macro_attribute]
pub fn obfuscate_api(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let path: syn::ItemConst = parse_macro_input!(input as syn::ItemConst);
    let path_name = &path.ident;
    let path_name_axum = format_ident!("{}_AXUM", path_name);

    let string_literal_error = || {
        syn::Error::new_spanned(&path.expr, "only string literals starting with '/' are supported")
            .to_compile_error()
            .into()
    };
    let path_string = match path.expr.as_ref() {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), ..}) => {
            let path_string = lit_str.value();
            if !path_string.starts_with('/') {
                return string_literal_error();
            }
            path_string
        }
        _ => {
            return string_literal_error();
        }
    };

    let new_path_string = obfuscate_path(&path_string);
    let new_path_string_axum = convert_to_axum_style_path(&new_path_string);
    let visibility = path.vis;

    let expanded = quote::quote! {
        #visibility const #path_name: &str = #new_path_string;
        #visibility const #path_name_axum: &str = #new_path_string_axum;
    };

    TokenStream::from(expanded)
}

fn obfuscate_path(path: &str) -> String {
    match path.split_once("/{") {
        Some((first, second)) => {
            format!(
                "/{}/{{{}",
                obfuscate(first),
                second,
            )
        }
        None => {
            format!(
                "/{}",
                obfuscate(path),
            )
        }
    }
}

fn convert_to_axum_style_path(path: &str) -> String {
    path.split('/')
        .map(|v| {
            if v.starts_with('{') && v.ends_with('}') {
                let param_name = v.trim_start_matches('{').trim_end_matches('}');
                format!(":{}", param_name)
            } else {
                v.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn obfuscate(text: &str) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}
