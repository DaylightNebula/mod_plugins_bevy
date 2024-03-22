use std::collections::HashMap;

use quote::quote;
use syn::{Attribute, Ident, ItemFn};

#[derive(Debug, Default, Clone)]
pub(crate) struct EventSystems {
    map: HashMap<Ident, Vec<Ident>>
}

impl EventSystems {
    pub(crate) fn push(&mut self, input: &mut ItemFn, attr: &Attribute) {
        // read event identifier
        let event = match &attr.meta {
            syn::Meta::List(list) => {
                let event = list.tokens.clone();
                syn::parse2(event).unwrap()
            },
            _ => panic!("Must be a meta list, example #[event(KeyboardInput)]")
        };

        // add current to function
        let current = quote! { current: Res<mod_plugins::resources::Current<#event>> };
        input.sig.inputs.push(syn::parse2(current).unwrap());

        // if we already have the event identifier in our map, simply add the system to the corresponding vector, 
        // otherwise, insert the event and create a new vector with the system
        if self.map.contains_key(&event) {
            self.map.get_mut(&event).unwrap().push(input.sig.ident.clone());
        } else {
            self.map.insert(event, vec![input.sig.ident.clone()]);
        }
    }

    pub(crate) fn append(&self, output: &mut proc_macro2::TokenStream, app_ext: &mut proc_macro2::TokenStream) {
        if !self.map.is_empty() {
            let mut event_groups = proc_macro2::TokenStream::new();
            for (event, vec) in self.map.iter() {
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
    }
}
