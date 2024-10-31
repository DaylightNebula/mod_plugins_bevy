use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Expr, Ident, ItemFn, Meta};

#[derive(Default)]
pub struct SystemProcessor {
    definitions: Vec<(FunctionDef, Ident)>,
    impl_functions: Vec<ItemFn>,
    base_functions: Vec<ItemFn>
}

enum FunctionDef {
    Impl,
    Build,
    ResourceSystem,
    ResourceFactory,
    System
}

impl SystemProcessor {
    pub fn process_item_fn(&mut self, mut item: ItemFn) {
        // define default function type
        let mut definition = FunctionDef::Impl;

        // run through each attribute to modify the existing function
        for attr in item.attrs.clone() {
            // get the attribute name (yes its ugly, welcome to Rust)
            let attr_name = attr.path().get_ident().unwrap();
            let attr_name = attr_name.to_string();
            let attr_name = attr_name.as_str();
            let tokens = meta_to_strings(attr.meta);

            // translate some attributes for backwards compatability
            let (attr_name, tokens) = match attr_name {
                "startup" => ("system", vec!["startup".to_string()]),
                "update" => ("system", vec!["update".to_string()]),
                "enter" => ("system", vec!["enter".to_string(), tokens[0].clone()]),
                "exit" => ("system", vec!["exit".to_string(), tokens[0].clone()]),
                "event" => {
                    let event = Ident::new(tokens[0].as_str(), Span::call_site());
                    item.attrs.push(syn::parse_quote! { #[event(#event)] });
                    ("system", vec!["update".to_string()])
                },
                _ => (attr_name, tokens)
            };

            // match attribute name to operation
            match attr_name {
                "system" => {
                    definition = build_system_enum_variant(&tokens);
                }

                "build" => {
                    definition = FunctionDef::Build;
                }

                "resource_factory" => {
                    definition = FunctionDef::ResourceFactory;
                }

                "resource_system" => {
                    definition = FunctionDef::ResourceSystem;
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
            FunctionDef::ResourceSystem => &mut self.base_functions,
            FunctionDef::ResourceFactory => &mut self.base_functions,
            FunctionDef::System => &mut self.base_functions
        };
        self.definitions.push((definition, item.sig.ident.clone()));
        item_list.push(item);
    }

    pub fn apply_app_exts(&self, app_exts: &mut TokenStream) { todo!() }
    pub fn apply_build(&self, builds: &mut TokenStream) { todo!() }

    pub fn impl_functions(&self) -> &[ItemFn] { return &self.impl_functions; }
    pub fn base_functions(&self) -> &[ItemFn] { return &self.base_functions; }
}

fn build_system_enum_variant(tokens: &[String]) -> FunctionDef {
    for token in tokens {
        match token.as_str() {
            _ => {}
        }
    }
    return FunctionDef::System;
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
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None
        }).collect();
}
