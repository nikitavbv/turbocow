extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Component)]
pub fn init_component(ts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);

    let struct_name = input.ident;
    let fields = match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);

    let res = quote! {
        impl livestonk::Resolve<#struct_name> for livestonk::Livestonk {
            fn resolve() -> Box<#struct_name> {
                box #struct_name {
                    #(
                        #field_name: livestonk::Livestonk::resolve(),
                    )*
                }
            }
        }
    }.to_string();

    res.parse().unwrap()
}