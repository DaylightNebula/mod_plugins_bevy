use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemMod};

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    // unpack
    let input = parse_macro_input!(input as ItemMod);

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
        parse_macro_input!(attr as Ident)
    };

    // setup some stuff for compute and output
    let mut output = proc_macro2::TokenStream::new();
    let mut startup: Vec<Ident> = Vec::new();
    let mut update: Vec<Ident> = Vec::new();
    let mut events: HashMap<Ident, Vec<Ident>> = HashMap::new();

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
                        let name = input.sig.ident.clone();

                        // match meta name to appropriate interpreting function, otherwise, skip
                        match meta_name {
                            // run this system on startup
                            "startup" => startup.push(name),

                            // run this system on update
                            "update" => update.push(name),

                            // run this system on event
                            "event" => {
                                // read event identifier
                                let event = match &attr.meta {
                                    syn::Meta::List(list) => {
                                        let event = list.tokens.clone();
                                        let event = event.into();
                                        parse_macro_input!(event as Ident)
                                    },
                                    _ => panic!("Must be a meta list, example #[event(KeyboardInput)]")
                                };

                                // add current to function
                                let current = quote! { current: Res<mod_plugins::resources::Current<#event>> };
                                let current = TokenStream::from(current);
                                input.sig.inputs.push(parse_macro_input!(current as FnArg));

                                // if we already have the event identifier in our map, simply add the system to the corresponding vector, 
                                // otherwise, insert the event and create a new vector with the system
                                if events.contains_key(&event) {
                                    events.get_mut(&event).unwrap().push(name);
                                } else {
                                    events.insert(event, vec![name]);
                                }
                            },
                            
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
    if !startup.is_empty() {
        app_ext.extend(quote! {
            .add_systems(bevy::prelude::Startup, ((#(#startup),*)))
        });
    }
    if !events.is_empty() {
        let mut event_groups = proc_macro2::TokenStream::new();
        for (event, vec) in events.iter() {
            // run through each system, run each event on each system
            let mut systems = proc_macro2::TokenStream::new();
            for system in vec {
                systems.extend(quote! {
                    {
                        for event in &events {
                            world.insert_resource(mod_plugins::resources::Current(event.clone()));

                            let mut system = IntoSystem::into_system(#system);
                            system.initialize(world);
                            system.run((), world);
                            system.apply_deferred(world);
                        }
                    }
                });
            }

            // read all events, run the above systems, and cleanup
            event_groups.extend(quote! {
                {
                    // read all events as a Vec<&{event}>
                    let events: Vec<#event> = {
                        let events = world.resource::<bevy::prelude::Events<#event>>();
                        let mut reader = events.get_reader();
                        reader.read(&events).map(|a| a.clone()).collect()
                    };

                    // run systems
                    #systems

                    world.remove_resource::<mod_plugins::resources::Current<#event>>();
                }
            });
        }

        // add _plugin_events system to output
        output.extend(quote! {
            fn _plugin_events(
                world: &mut World
            ) {
                #event_groups
            }
        });

        // add _plugin_events as update system
        app_ext.extend(quote! {
            .add_systems(bevy::prelude::Update, _plugin_events)
        });
    }
    if !update.is_empty() {
        app_ext.extend(quote! {
            .add_systems(bevy::prelude::Update, ((#(#update),*)))
        });
    }

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