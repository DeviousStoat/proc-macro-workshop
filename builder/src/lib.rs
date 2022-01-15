use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, Ident, Path, PathSegment, TypePath, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let builder_struct_name = format_ident!("{struct_name}Builder");

    let struct_fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => unimplemented!()
        }
        _ => unimplemented!()
    };

    let struct_fields_names: Vec<_> = struct_fields.iter().map(|f| &f.ident).collect();
    let struct_fields_types: Vec<_> = struct_fields.iter().map(|f| &f.ty).collect();

    let (optional_struct_fields, other_struct_fields): (Vec<_>, Vec<_>) = struct_fields
        .iter()
        .partition(|f| is_optional(&f.ty));

    let other_struct_fields_names: Vec<_> = other_struct_fields.iter().map(|f| &f.ident).collect();
    let optional_struct_fields_names: Vec<_> = optional_struct_fields.iter().map(|f| &f.ident).collect();

    let builder_struct = quote! {
        pub struct #builder_struct_name {
            #(#other_struct_fields_names: Option<#struct_fields_types>,)*
            #(#optional_struct_fields_names: #struct_fields_types,)*
        }
    };

    let new = quote! {
        #builder_struct

        impl #struct_name {
            pub fn builder() -> #builder_struct_name {
                #builder_struct_name {
                    #(#struct_fields_names: None,)*
                }
            }
        }

        impl #builder_struct_name {
            #(
                pub fn #struct_fields_names(&mut self, #struct_fields_names: #struct_fields_types) -> &mut Self {
                    self.#struct_fields_names = Some(#struct_fields_names);
                    self
                }
            )*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#optional_struct_fields_names: self.#optional_struct_fields_names.take(),)*
                    #(#other_struct_fields_names: self.#other_struct_fields_names.take().ok_or("missing field")?,)*
                })
            }
        }
    };

    TokenStream::from(new)
}

fn is_optional(ty: &Type) -> bool {
    // let ident_to_match = format_ident!("Option");

    let segments = match ty {
        Type::Path(TypePath {
            path,
            qself: None,
            ..
        }) => &path.segments,
        _ => return false,
    };

    segments.len() == 1 && segments[0].ident == "Option"
}
