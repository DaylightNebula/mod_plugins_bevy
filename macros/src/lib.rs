use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemMod};

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    // unpack
    let input = parse_macro_input!(input as ItemMod);
    let struct_name = if attr.is_empty() {
        let name = input.ident.to_string();
        let struct_name = name.to_string()
            .split("_")
            .map(|token| {
                let mut chars: Vec<char> = token.chars().collect();
                chars[0] = chars[0].to_uppercase().nth(0).unwrap();
                chars.into_iter().collect()
            }).collect::<Vec<String>>().join("");
        Ident::new(struct_name.as_str(), Span::call_site())
    } else {
        parse_macro_input!(attr as Ident)
    };
    let mut output = TokenStream::new();

    let mut startup: Vec<Ident> = Vec::new();

    // assemble initial output
    for input in input.content.unwrap().1 {
        match input {
            // add function to output while using metadata to interpret if and which type of system this function is
            syn::Item::Fn(mut input) => {
                // for each attribute on the function, check its metadata from its identifier
                for attr in &input.attrs {
                    if let Some(meta_name) = attr.path().get_ident() {
                        // get metadata name
                        let meta_name = meta_name.to_string();
                        let meta_name = meta_name.as_str();

                        // match meta name to appropriate interpreting function, otherwise, skip
                        match meta_name {
                            // run this system on startup
                            "startup" => {
                                let name = input.sig.ident.clone();
                                startup.push(name);
                            },
                            
                            _ => {}
                        }
                    }
                }

                // add the function to the output
                input.attrs.clear();
                output.extend(TokenStream::from(quote! { #input }))
            },

            // syn::Item::Struct(_) => todo!(),
            // syn::Item::Use(_) => todo!(),
            
            // by default, just add to the output
            _ => output.extend(TokenStream::from(quote! { #input }))
        }
    }

    output.extend(TokenStream::from(quote! {
        pub struct #struct_name;
        impl bevy::prelude::Plugin for #struct_name {
            fn build(&self, app: &mut bevy::prelude::App) {
                app.add_systems(bevy::prelude::Startup, #(#startup),*);
            }
        }
    }));

    output
}