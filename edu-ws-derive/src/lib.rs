//! Proc macros for edu-ws.

#![warn(rust_2018_idioms)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![deny(rustdoc::all)]

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(HexWrapper)]
pub fn hex_wrapper_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (_, inner) = inner(&input.data);
    let inner_expr = inner_expr(inner);
    let debug_impl = match inner {
        Some(_) => quote! {
            f.debug_struct(stringify!(#name))
                .field(stringify!(#inner_expr), &format!("{:x}", self))
                .finish()
        },
        None => quote! {
            f.debug_tuple(stringify!(#name))
                .field(&format!("{:x}", self))
                .finish()
        },
    };
    let expanded = quote! {
        impl #name {
            pub fn parse(s: &str) -> Result<Self, hex::FromHexError> {
                let raw = hex::FromHex::from_hex(s)?;
                Ok(Self{ #inner_expr: raw })
            }
        }

        impl std::fmt::LowerHex for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", hex::encode(self.#inner_expr))
            }
        }

        impl std::fmt::UpperHex for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", hex::encode_upper(self.#inner_expr))
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #debug_impl
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:x}", self)
            }
        }

        impl std::str::FromStr for #name {
            type Err = hex::FromHexError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                #name::parse(s)
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(DerefWrapper)]
pub fn deref_wrapper_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (ty, inner) = inner(&input.data);
    let inner_expr = inner_expr(inner);
    let expanded = quote! {
        impl std::ops::Deref for #name {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {
                &self.#inner_expr
            }
        }

        impl std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#inner_expr
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(FromWrapper)]
pub fn from_wrapper_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (ty, inner) = inner(&input.data);
    let inner_expr = inner_expr(inner);
    let expanded = quote! {
        impl From<#name> for #ty {
            fn from(wrapper: #name) -> Self {
                wrapper.#inner_expr
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn inner(data: &Data) -> (&Type, &Option<Ident>) {
    let fields = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Unnamed(ref fields) => &fields.unnamed,
            Fields::Named(ref fields) => &fields.named,
            Fields::Unit => unimplemented!("struct needs to have exactly one field"),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!("can only derive for structs"),
    };

    if fields.len() != 1 {
        unimplemented!("struct needs to have exactly one field");
    }
    let field = &fields.first().unwrap();
    (&field.ty, &field.ident)
}

fn inner_expr(inner: &Option<Ident>) -> proc_macro2::TokenStream {
    match inner {
        Some(ident) => quote! { #ident },
        None => quote! { 0 },
    }
}
