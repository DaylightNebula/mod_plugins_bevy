use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Expr, FnArg, Ident, ItemFn, Meta, Pat, ReturnType};
use quote::quote;

#[derive(Default)]
pub struct SystemProcessor {
    // definitions: Vec<(FunctionDef, Ident)>,
    definitions: HashMap<Ident, FunctionDef>,
    impl_functions: Vec<ItemFn>,
    base_functions: Vec<ItemFn>
}

enum FunctionDef {
    Impl,
    Build,
    ResourceFactory,
    System(Expr, Priority)
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)]
enum Priority {
    LOWEST,
    LOW,
    NORMAL,
    HIGH,
    HIGHEST,
    CUSTOM(u32)
}

impl SystemProcessor {
    pub fn process_item_fn(&mut self, mut item: ItemFn) {
        // define default function type
        let mut definition = FunctionDef::Impl;

        // run through each attribute to modify the existing function
        for attr in item.attrs.clone() {
            // get the attribute name (yes its ugly, welcome to Rust)
            let attr_name = attr.path().get_ident().expect("Failed to unwrap attr_name");
            let attr_name = attr_name.to_string();
            let attr_name = attr_name.as_str();
            let tokens = meta_to_strings(attr.meta);

            // translate some attributes for backwards compatability
            let (attr_name, tokens) = match attr_name {
                "startup" => ("system", vec!["startup".to_string()]),
                "update" => ("system", vec!["update".to_string()]),

                "enter" => {
                    let mut vec = vec!["enter".to_string()];
                    vec.extend(tokens);
                    ("system", vec)
                },

                "exit" => {
                    let mut vec = vec!["exit".to_string()];
                    vec.extend(tokens);
                    ("system", vec)
                },
                _ => (attr_name, tokens)
            };

            // match attribute name to operation
            match attr_name {
                "build" => { definition = FunctionDef::Build; }
                "resource_factory" => { definition = FunctionDef::ResourceFactory; }

                "resource_system" => { 
                    // add system definition
                    definition = FunctionDef::System(
                        syn::parse2(quote! { Startup }).expect("Failed to unwrap Update arg."), 
                        Priority::NORMAL
                    );

                    // make sure we have a commands argument
                    let commands = item.sig.inputs.iter().filter_map(|arg| {
                        if let FnArg::Typed(pat) = arg {
                            if pat.ty == syn::parse2(quote! { Commands }).unwrap() {
                                if let Pat::Ident(ident) = *pat.pat.to_owned() {
                                    Some(ident.ident)
                                } else { None }
                            } else { None }
                        } else { None }
                    }).next();

                    // get or create commands
                    let commands = if commands.is_some() { 
                        commands.unwrap() 
                    } else {
                        item.sig.inputs.push(syn::parse2(quote! {
                            mut commands: Commands
                        }).unwrap());
                        Ident::new("commands", Span::call_site())
                    };

                    // remove return
                    item.sig.output = ReturnType::Default;

                    // add code to add resource
                    let block = item.block;
                    item.block = syn::parse2(quote! {
                        {
                            let resource = #block;
                            #commands.insert_resource(resource);
                        }
                    }).unwrap();
                }

                "system" => {
                    definition = build_system_enum_variant(&tokens);
                    let exec = tokens[0].to_string();
                    let exec = exec.as_str();

                    if exec == "enter" || exec == "exit" {
                        let input = tokens[1].to_string();
                        let input = Ident::new(input.as_str(), Span::call_site());
                        item.sig.inputs.push(syn::parse2(quote! {
                            current: Res<State<#input>>
                        }).unwrap());
                    }
                }

                "event" => {
                    // if def has not been set yet, set to update
                    if matches!(definition, FunctionDef::Impl) {
                        definition = FunctionDef::System(
                            syn::parse2(quote! { Update }).expect("Failed to unwrap Update arg."), 
                            Priority::NORMAL
                        );
                    }

                    // get event names and argument names
                    let event_name = tokens[0].clone();
                    let event_arg = Ident::new(event_name.to_case(Case::Snake).as_str(), Span::call_site());
                    let event = Ident::new(event_name.as_str(), Span::call_site());
                    
                    // add argument for event
                    item.sig.inputs.push(syn::parse2(quote! {
                        mut #event_arg: EventReader<#event>
                    }).expect("Failed to unwrap event argument."));

                    // add test for event
                    let block = item.block;
                    item.block = syn::parse2(quote! {
                        {
                            for #event_arg in #event_arg.read() {
                                #block
                            }
                        }
                    }).expect("Failed to unwrap event block.");
                }

                "priority" => match definition {
                    FunctionDef::System(_, ref mut prio) => { 
                        *prio = match tokens[0].as_str() {
                            "LOWEST" => Priority::LOWEST,
                            "LOW" => Priority::LOW,
                            "NORMAL" => Priority::NORMAL,
                            "HIGH" => Priority::HIGH,
                            "HIGHEST" => Priority::HIGHEST,
                            "CUSTOM" => Priority::CUSTOM(str::parse(tokens[1].as_str()).expect("Could not parse i32")),
                            _ => panic!("Invalid priority")
                        } 
                    },
                    _ => panic!("Priority attribute can only be applied to systems!")
                }

                _ => panic!("Unknown plugin attribute {:?}", attr_name)
            }
        }

        // remove all attributes
        item.attrs.clear();

        // save definiton and function item
        let item_list = match &definition {
            FunctionDef::Impl => &mut self.impl_functions,
            FunctionDef::Build => &mut self.impl_functions,
            FunctionDef::ResourceFactory => &mut self.base_functions,
            FunctionDef::System(_, _) => &mut self.base_functions
        };
        self.definitions.insert(item.sig.ident.clone(), definition);
        item_list.push(item);
    }

    pub fn apply_app_exts(&mut self, app_exts: &mut TokenStream) {
        // sort systems by expression
        let mut systems = HashMap::<Expr, Vec<(Priority, Ident)>>::new();

        // for each definition, add only systems
        for (name, def) in self.definitions.iter_mut() {
            match def {
                // add systems to systems tracking map
                FunctionDef::System(expr, priority) => {
                    if systems.contains_key(&expr) {
                        systems.get_mut(&expr).expect("Failed to unwrap systems get.").push((*priority, name.clone()));
                    } else {
                        systems.insert(expr.clone(), vec![(*priority, name.clone())]);
                    }
                },

                _ => {}
            }
        }

        // sort and add each systems list to app extensions
        for (expr, mut vec) in systems.drain() {
            vec.sort_by_key(|(prio, _)| match prio {
                Priority::LOWEST => u32::MAX,
                Priority::LOW => u32::MAX / 4 * 3,
                Priority::NORMAL => u32::MAX / 2,
                Priority::HIGH => u32::MAX / 4,
                Priority::HIGHEST => u32::MIN,
                Priority::CUSTOM(prio) => u32::MAX - *prio
            });
            let vec = vec.into_iter().map(|(_, a)| a).collect::<Vec<_>>();
            app_exts.extend(quote!{ .add_systems(#expr, (#(#vec),*).chain()) });
        }

        // add resource factory results
        let factories = self.definitions.iter()
            .filter_map(|(factory, def)| match def {
                FunctionDef::ResourceFactory => Some(factory),
                _ => None
            });
        for factory in factories {
            app_exts.extend(quote! { .insert_resource(#factory()) });
        }
    }

    pub fn apply_build(&self, builds: &mut TokenStream) {
        let build_funcs = self.definitions
            .iter()
            .filter_map(|(ident, def)| match def {
                FunctionDef::Build => Some(ident.clone()),
                _ => None
            }).collect::<Vec<_>>();
        builds.extend(quote! { #(self.#build_funcs(app);)* })
    }

    pub fn impl_functions(&self) -> &[ItemFn] { return &self.impl_functions; }
    pub fn base_functions(&self) -> &[ItemFn] { return &self.base_functions; }
}

fn build_system_enum_variant(tokens: &[String]) -> FunctionDef {
    let expr: syn::Expr = match tokens[0].as_str() {
        "update" => syn::parse2(quote! { Update }).expect("Failed to unwrap Update system expr."),
        "startup" => syn::parse2(quote! { Startup }).expect("Failed to unwrap Startup system expr."),
        "enter" => {
            let state = tokens[1..].join("");
            let state: Expr = syn::parse2(state.parse().unwrap()).unwrap();
            syn::parse2(quote! { OnEnter(#state) }).expect("Failed to unwrap OnEnter system expr.")
        }
        "exit" => {
            let state = tokens[1..].join("");
            let state: Expr = syn::parse2(state.parse().unwrap()).unwrap();
            syn::parse2(quote! { OnExit(#state) }).expect("Failed to unwrap OnExit system expr.")
        }
        _ => panic!("Unknown build system type {:?}", tokens[0])
    };
    return FunctionDef::System(expr, Priority::NORMAL);
}

fn meta_to_strings(meta: Meta) -> Vec<String> {
    match meta {
        Meta::List(list) => tokens_to_strings(list.tokens),
        _ => vec![]
    }
}

fn tokens_to_strings(tokens: TokenStream) -> Vec<String> {
    return tokens
        .into_iter()
        .filter_map(|a| match a {
            TokenTree::Ident(ident) => Some(vec![ident.to_string()]),
            TokenTree::Literal(literal) => Some(vec![literal.to_string()]),
            TokenTree::Group(group) => Some(tokens_to_strings(group.stream())),
            TokenTree::Punct(punct) => Some(vec![punct.to_string()])
        }).flatten().collect();
}
