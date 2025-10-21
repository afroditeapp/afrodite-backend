use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Fields, Lit};

#[proc_macro_derive(SimpleDieselEnum)]
pub fn simple_diesel_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let data = &input.data;

    let variants = match data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("SimpleDieselEnum can only be derived for enums"),
    };

    let mut to_sql_arms = vec![];

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            panic!("SimpleDieselEnum only supports unit variants");
        }

        let ident = &variant.ident;
        if let Some((_, expr)) = &variant.discriminant {
            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                let value: i16 = lit_int.base10_parse().expect("Discriminant must be a valid i16");
                to_sql_arms.push(quote! {
                    #name::#ident => #value.to_sql(out),
                });
            } else {
                panic!("Discriminant must be an integer literal");
            }
        } else {
            panic!("Each variant must have a discriminant");
        }
    }

    let expanded = quote! {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB> for #name
        where
            i16: diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i16::from_sql(value)?;
                TryInto::<#name>::try_into(value).map_err(|e| e.into())
            }
        }

        impl<DB> diesel::serialize::ToSql<diesel::sql_types::SmallInt, DB> for #name
        where
            DB: diesel::backend::Backend,
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, DB>,
        {
            fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
                match *self {
                    #(#to_sql_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
