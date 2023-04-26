extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(View)]
pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let data = match data {
        Data::Struct(data) => data,
        _ => panic!("Derive can only be used on struct types"),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields_name = format_ident!("{}Fields", ident);

    let expanded_fields_name = view_fields(&fields_name, &data);

    let expanded = quote! {
        impl #impl_generics View for #ident #ty_generics #where_clause {
            type Fields = #fields_name;
        }

        #expanded_fields_name

        impl core::fmt::Display for #fields_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                use serde_view::ViewFields;
                f.write_str(self.as_str())
           }
        }

    };

    proc_macro::TokenStream::from(expanded)
}

fn view_fields(name: &Ident, data: &DataStruct) -> TokenStream {
    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        _ => {
            panic!("Derive can only be used on a struct with named fields");
        }
    };

    let fields = fields
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .map(|name| {
            (
                name.to_string(),
                Ident::new(&name.to_string().to_case(Case::Pascal), name.span()),
            )
        })
        .collect::<Vec<_>>();

    let variants = fields.iter().map(|(_, variant)| {
        quote! {
            #variant
        }
    });
    let as_str_impl = fields.iter().map(|(name, variant)| {
        quote! {
            Self::#variant => #name
        }
    });
    let from_str_impl_1 = fields.iter().map(|(name, variant)| {
        quote! {
            #name => Self::#variant
        }
    });
    let from_str_impl_2 = fields.iter().map(|(name, variant)| {
        quote! {
            #name => Self::#variant
        }
    });

    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum #name {
            #(#variants, )*
        }

        impl serde_view::ViewFields for #name {

            fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_impl, )*
                }
            }

            fn from_str(name: &str) -> serde_view::Result<Self> {
                Ok(match name {
                    #(#from_str_impl_1, )*
                    s => return Err(serde_view::Error::UnknownField(s.to_string())),
                })
            }

        }

        impl std::str::FromStr for #name {
            type Err = serde_view::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    #(#from_str_impl_2, )*
                    s => return Err(serde_view::Error::UnknownField(s.to_string())),
                })
            }
        }
    }
}
