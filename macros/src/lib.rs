use convert_case::Casing;
use initialization::InitializationSystems;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Fields, FnArg, Ident, ItemFn, ItemMod, ItemStruct, Token, Type};
use systems::SystemProcessor;

mod initialization;
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
    let mut fields = Vec::<syn::Field>::new();
    let mut default_resources = Vec::<syn::Ident>::new();

    // assemble initial output
    for input in input.content.unwrap().1 {
        match input {
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
                                default_resources.push(struct_item.ident.clone());
                                false
                            },

                            "init_event" => {
                                init.events.push(struct_item.ident.clone());
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
                let mut extras = proc_macro2::TokenStream::new();

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
                                default_resources.push(enum_item.ident.clone());
                                false
                            },

                            "init_event" => {
                                init.events.push(enum_item.ident.clone());
                                false
                            }

                            "init_state" => {
                                // add state to definitions lists
                                if let Ok(list) = attr.meta.require_list() {
                                    init.states_nodef.push(syn::parse2(list.tokens.clone()).unwrap());
                                } else {
                                    init.states_def.push(enum_item.ident.clone());
                                }

                                // add derive macros
                                extras.extend(quote! {
                                    #[derive(States, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
                                });

                                // add system for each variant
                                let ident = &enum_item.ident;
                                enum_item.variants.iter().for_each(|variant| {
                                    let variant = &variant.ident;
                                    let func_name = Ident::new(format!("_remove_{}", variant.to_string()).to_case(convert_case::Case::Snake).as_str(), Span::call_site());
                                    let enter_function = quote! {
                                        #[enter(#ident::#variant)]
                                        fn #func_name(
                                            mut commands: bevy::prelude::Commands,
                                            mut query: bevy::prelude::Query<(bevy::prelude::Entity, &mod_plugins::resources::ScopeLocal<#ident>)>
                                        ) {
                                            query.iter().for_each(|(entity, scope)| {
                                                if (scope.0 != #ident::#variant) {
                                                    commands.entity(entity).despawn_recursive();
                                                }
                                            });
                                        }
                                    };
                                    systems.process_item_fn(syn::parse2(enter_function).expect("Could not compile enter remove state function."));
                                });

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

                output.extend(quote! { 
                    #extras 
                    #enum_item 
                })
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
    let mut builds = proc_macro2::TokenStream::new();
    let mut app_ext = proc_macro2::TokenStream::new();
    let mut impl_funcs = proc_macro2::TokenStream::new();
    let mut base_funcs = proc_macro2::TokenStream::new();
    init.append(&mut app_ext);

    // apply systems
    systems.apply_build(&mut builds);
    systems.apply_app_exts(&mut app_ext);
    for impl_func in systems.impl_functions().iter() {
        impl_funcs.extend(quote! { #impl_func });
    }
    for base_func in systems.base_functions().iter() {
        base_funcs.extend(quote! { #base_func });
    } 
    for default_res in default_resources.iter() {
        app_ext.extend(quote! { .insert_resource(#default_res::default()) });
    }

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
                #builds
                
                app #app_ext;
            }
        }

        impl #struct_name {
            #impl_funcs
        }

        #base_funcs
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

#[proc_macro_attribute]
pub fn prefab(attr: TokenStream, input: TokenStream) -> TokenStream {
    // parse inputs
    let mut input = parse_macro_input!(input as ItemStruct);
    let tokens = systems::tokens_to_strings(attr.into());

    // setup scope if given and using named fields
    if let Fields::Named(fields) = &mut input.fields {
        // get index of scope attribute and check if it was given
        let scope_idx = tokens.iter()
            .enumerate()
            .find(|a| a.1 == "scope")
            .map(|a| a.0);
        if let Some(scope_idx) = scope_idx {
            // get the scope token and add
            if let Some(scope) = tokens.iter().nth(scope_idx + 1) {
                // add field based on scope given
                match scope.as_str() {
                    // add global field
                    "global" => {
                        fields.named.push(syn::Field {
                            attrs: vec![],
                            vis: syn::Visibility::Public(<Token![pub]>::default()),
                            mutability: syn::FieldMutability::None,
                            ident: Some(Ident::new("scope", Span::call_site())),
                            colon_token: Some(<Token![:]>::default()),
                            ty: syn::parse2(quote! { mod_plugins::resources::ScopeGlobal }).unwrap()
                        });
                    },

                    // add local field
                    "local" => {
                        let state = Ident::new(tokens[scope_idx + 2].as_str(), Span::call_site());
                        fields.named.push(syn::Field {
                            attrs: vec![],
                            vis: syn::Visibility::Public(<Token![pub]>::default()),
                            mutability: syn::FieldMutability::None,
                            ident: Some(Ident::new("scope", Span::call_site())),
                            colon_token: Some(<Token![:]>::default()),
                            ty: syn::parse2(quote! { mod_plugins::resources::ScopeLocal<#state> }).unwrap()
                        })
                    },

                    // panic if unknown scope
                    _ => panic!("Unknown scope {scope:?}")
                }
            }
        }
    }

    // compile and return result
    TokenStream::from(quote! {
        #[derive(Bundle, Default)]
        #input
    })
}
