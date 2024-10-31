use basic::BasicSystems;
use events::EventSystems;
use initialization::InitializationSystems;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use resources::ResourceSystems;
use states::{StateSystems, StateType};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ItemMod, Type};
use systems::SystemProcessor;

mod basic;
mod events;
mod initialization;
mod resources;
mod states;
mod systems;

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
    let mut init = InitializationSystems::default();
    let mut systems = SystemProcessor::default();
    // let mut basics = BasicSystems::default();
    // let mut events = EventSystems::default();
    // let mut states = StateSystems::default();
    // let mut resources = ResourceSystems::default();
    // let mut build_funcs = Vec::<syn::Ident>::new();
    let mut fields = Vec::<syn::Field>::new();
    // let mut std_impl = proc_macro2::TokenStream::new();

    // assemble initial output
    for input in input.content.unwrap().1 {
        match input {
            // // add function to output while using metadata to interpret if and which type of system this function is
            // syn::Item::Fn(mut input) => {
            //     // for each attribute on the function, check its metadata from its identifier
            //     let mut add_to_std_impl = false;
            //     for attr in input.attrs.clone() {
            //         if let Some(meta_name) = attr.path().get_ident() {
            //             // get metadata name
            //             let name = input.sig.ident.clone();
            //             let meta_name = meta_name.to_string();
            //             let meta_name = meta_name.as_str();

            //             // match meta name to appropriate interpreting function, otherwise, skip
            //             match meta_name {
            //                 "startup" => basics.push_startup(name),
            //                 "update" => basics.push_update(name),
            //                 "event" => events.push(&mut input, &attr),
            //                 "enter" => states.push(StateType::Enter, &attr, name),
            //                 "exit" => states.push(StateType::Exit, &attr, name),
            //                 "resource_factory" => resources.push_factory(name),
            //                 "resource_system" => resources.push_system(name),
            //                 "build" => {
            //                     add_to_std_impl = true; 
            //                     build_funcs.push(name)
            //                 },
                            
            //                 _ => {}
            //             }
            //         }
            //     }

            //     // add the function to the output
            //     input.attrs.clear();
            //     if !add_to_std_impl { output.extend(quote! { #input }); }
            //     else { std_impl.extend(quote! { #input }); }
            // },

            syn::Item::Fn(item) => systems.process_item_fn(item),

            syn::Item::Struct(mut struct_item) => {
                // go through each attribute, choosing whether we should keep it or not (Rust how no idea what our custom attributes are and will cancel)
                struct_item.attrs.retain(|attr| {
                    // attempt to get the attributes path as its identifier, otherwise, return keep
                    if let Some(meta_name) = attr.path().get_ident() {
                        // unpack attribute metadata name
                        let meta_name = meta_name.to_string();
                        let meta_name = meta_name.as_str();

                        // attempt to match attribute name, otherwise, return keep
                        match meta_name {
                            // make the resource initialize by its default in the App
                            "init_resource" => {
                                // resources.push_default(struct_item.ident.clone());
                                false
                            },

                            "init_event" => {
                                init.events.push(struct_item.ident.clone());
                                false
                            }

                            "init_state" => {
                                if let Ok(list) = attr.meta.require_list() {
                                    init.states_nodef.push(syn::parse2(list.tokens.clone()).unwrap());
                                } else {
                                    init.states_def.push(struct_item.ident.clone());
                                }

                                false
                            }

                            "register" => {
                                init.registered.push(struct_item.ident.clone());
                                false
                            }

                            _ => true
                        }
                    } else { true }
                });

                output.extend(quote! { #struct_item })
            },

            syn::Item::Enum(mut enum_item) => {
                // go through each attribute, choosing whether we should keep it or not (Rust how no idea what our custom attributes are and will cancel)
                enum_item.attrs.retain(|attr| {
                    // attempt to get the attributes path as its identifier, otherwise, return keep
                    if let Some(meta_name) = attr.path().get_ident() {
                        // unpack attribute metadata name
                        let meta_name = meta_name.to_string();
                        let meta_name = meta_name.as_str();

                        // attempt to match attribute name, otherwise, return keep
                        match meta_name {
                            // make the resource initialize by its default in the App
                            "init_resource" => {
                                // resources.push_default(enum_item.ident.clone());
                                false
                            },

                            "init_event" => {
                                init.events.push(enum_item.ident.clone());
                                false
                            }

                            "init_state" => {
                                if let Ok(list) = attr.meta.require_list() {
                                    init.states_nodef.push(syn::parse2(list.tokens.clone()).unwrap());
                                } else {
                                    init.states_def.push(enum_item.ident.clone());
                                }

                                false
                            }

                            "register" => {
                                init.registered.push(enum_item.ident.clone());
                                false
                            }

                            _ => true
                        }
                    } else { true }
                });

                output.extend(quote! { #enum_item });
            },

            syn::Item::Type(type_item) => {
                let mut passthrough = true;

                if !type_item.attrs.is_empty() {
                    if let Some(meta_name) = (&type_item.attrs[0]).path().get_ident() {
                        match meta_name.to_string().as_str() {
                            "field" => {
                                passthrough = false;
                                let vis = &type_item.vis;
                                let ident = &type_item.ident;
                                let ty = &type_item.ty;
                                fields.push(syn::Field {
                                    attrs: vec![],
                                    vis: vis.clone(),
                                    mutability: syn::FieldMutability::None,
                                    ident: Some(ident.clone()),
                                    colon_token: Some(syn::token::Colon::default()),
                                    ty: *ty.clone()
                                });
                            }

                            _ => {}
                        }
                    }
                }

                if passthrough { output.extend(quote! { #type_item }) }
            }
            
            // by default, just add to the output
            _ => {
                output.extend(quote! { #input });
            }
        }
    }

    // compile app extensions
    let mut app_ext = proc_macro2::TokenStream::new();
    // basics.append(&mut app_ext);
    init.append(&mut app_ext);
    // states.append(&mut app_ext);
    // events.append(&mut output, &mut app_ext);
    // resources.append(&mut output, &mut app_ext);

    // compile after struct
    let after_struct = if fields.is_empty() { quote! { ; } } else {
        quote! {
            {
                #(#fields),*
            }
        }
    };

    // compile final plugin output
    output.extend(quote! {
        pub struct #struct_name #after_struct
        impl bevy::prelude::Plugin for #struct_name {
            fn build(&self, app: &mut bevy::prelude::App) {
                // #(self.#build_funcs(app);)*
                
                app #app_ext;
            }
        }

        impl #struct_name {
            // #std_impl
        }
    });

    output.into()
}

#[proc_macro_attribute]
pub fn executable(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(attr as Ident);
    let mut func = parse_macro_input!(input as ItemFn);
    let sig = &mut func.sig;
    let name = &sig.ident;

    // add current arg
    let current = quote! { current: Res<mod_plugins::resources::Current<Box<#ident>>> };
    let current = TokenStream::from(current);
    sig.inputs.push(parse_macro_input!(current as FnArg));

    // get return type with hacky workaround for ()
    let empty = Box::new(Type::Verbatim(quote! { () }));
    let ret = match &sig.output {
        syn::ReturnType::Default => &empty,
        syn::ReturnType::Type(_, b) => b,
    };

    TokenStream::from(quote! {
        impl mod_plugins::resources::Executable<#ret> for #ident {
            fn execute(self: Box<Self>, world: &mut bevy::prelude::World) -> #ret {
                let mut system = bevy::prelude::IntoSystem::into_system(#name);
                world.insert_resource(mod_plugins::resources::Current::new(self));
                system.initialize(world);
                let response = system.run((), world);
                system.apply_deferred(world);
                world.remove_resource::<mod_plugins::resources::Current<Self>>();
        
                response
            }
        }

        #func
    })
}
