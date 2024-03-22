use std::collections::HashMap;

use basic::BasicSystems;
use events::EventSystems;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use states::{StateSystems, StateType};
use syn::{Ident, ItemMod};

mod basic;
mod events;
mod states;

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    // unpack
    let input: ItemMod = syn::parse(input).unwrap();

    // get structure name, if no structure name given, generate from the module name
    let struct_name = if attr.is_empty() {
        // return structure name where the modules name is converted from snake case to cammel case
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
        // return the structure name given
        syn::parse(attr).unwrap()
    };

    // setup some stuff for compute and output
    let mut output = proc_macro2::TokenStream::new();
    let mut basics = BasicSystems::default();
    let mut events = EventSystems::default();
    let mut states = StateSystems::default();

    // assemble initial output
    for input in input.content.unwrap().1 {
        match input {
            // add function to output while using metadata to interpret if and which type of system this function is
            syn::Item::Fn(mut input) => {
                // for each attribute on the function, check its metadata from its identifier
                for attr in input.attrs.clone() {
                    if let Some(meta_name) = attr.path().get_ident() {
                        // get metadata name
                        let name = input.sig.ident.clone();
                        let meta_name = meta_name.to_string();
                        let meta_name = meta_name.as_str();

                        // match meta name to appropriate interpreting function, otherwise, skip
                        match meta_name {
                            "startup" => basics.push_startup(name),
                            "update" => basics.push_update(name),
                            "event" => events.push(&mut input, &attr),
                            "enter" => states.push(StateType::Enter, &attr, name),
                            "exit" => states.push(StateType::Exit, &attr, name),
                            
                            _ => {}
                        }
                    }
                }

                // add the function to the output
                input.attrs.clear();
                output.extend(quote! { #input })
            },

            // syn::Item::Struct(_) => todo!(),
            // syn::Item::Use(_) => todo!(),
            
            // by default, just add to the output
            _ => output.extend(quote! { #input })
        }
    }

    // compile app extensions
    let mut app_ext = proc_macro2::TokenStream::new();
    basics.append(&mut app_ext);
    states.append(&mut app_ext);
    events.append(&mut output, &mut app_ext);

    // compile final plugin output
    output.extend(quote! {
        pub struct #struct_name;
        impl bevy::prelude::Plugin for #struct_name {
            fn build(&self, app: &mut bevy::prelude::App) {
                app #app_ext;
            }
        }
    });

    output.into()
}