//! Macros for use with the AcaciaLinux tooling project

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Expr, Meta};

/// A error in a derive macro
macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

/// A derive macro that implements the `ReprU16` trait for an enum
#[proc_macro_derive(IntoU16, attributes(expose))]
pub fn repr_u16(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let mut found = false;
    for x in input.attrs.clone() {
        if !x.path().is_ident("repr") {
            continue;
        }

        match x.meta {
            Meta::List(list) => {
                if &list.tokens.to_string() == "u16" {
                    found = true;
                }
            }
            _ => {}
        }
    }

    if !found {
        return derive_error!("Need enum to use #[repr(u16)]");
    }

    let enum_ident = input.ident;
    match input.data.clone() {
        Data::Enum(data) => {
            let mut fields = Vec::new();

            for variant in data.variants {
                let variant_ident = variant.ident;

                let disciminant = if let Some(d) = variant.discriminant {
                    d
                } else {
                    return derive_error!("ALL enum variants need an explicitly assigned value!");
                };

                let literal = if let Expr::Lit(e) = disciminant.1 {
                    e.lit
                } else {
                    return derive_error!("ALL enum variants need a literal value");
                };

                fields.push(quote! {
                    #literal => Some(#enum_ident::#variant_ident),
                });
            }

            quote! {
                impl ReprU16 for #enum_ident {
                    fn into_u16(&self) -> u16 {
                        unsafe { *(self as *const Self as *const u16) }
                    }

                    fn from_u16(num: u16) -> Option<Self> {
                        match num {
                            #(#fields)*
                            _ => None,
                        }
                    }
                }
            }
            .into()
        }
        _ => derive_error!("Can only implement IntoU16 for enums"),
    }
}
