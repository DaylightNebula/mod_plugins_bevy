use std::collections::HashMap;

use quote::*;
use syn::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct StateSystems {
    enters: HashMap<Expr, Vec<Ident>>,
    exits: HashMap<Expr, Vec<Ident>>
}

pub(crate) enum StateType {
    Enter,
    Exit
}

impl StateSystems {
    pub(crate) fn push(&mut self, ty: StateType, attr: &Attribute, name: Ident) {
        // get kind
        let kind: Expr = match &attr.meta {
            syn::Meta::List(list) => {
                let kind = list.tokens.clone();
                parse2(kind).unwrap()
            },
            _ => panic!("Must be a meta list, example #[event(KeyboardInput)]")
        };

        // get map to append too
        let map = match ty {
            StateType::Enter => &mut self.enters,
            StateType::Exit => &mut self.exits
        };

        // push or insert into map
        if map.contains_key(&kind) {
            map.get_mut(&kind).unwrap().push(name);
        } else {
            map.insert(kind, vec![name]);
        }
    }

    pub fn append(&self, app_ext: &mut proc_macro2::TokenStream) {
        if !self.enters.is_empty() {
            for (state, systems) in &self.enters {
                app_ext.extend(quote! {
                    .add_systems(bevy::prelude::OnEnter(#state), ((#(#systems),*)))
                });
            }
        }
        if !self.exits.is_empty() {
            for (state, systems) in &self.exits {
                app_ext.extend(quote! {
                    .add_systems(bevy::prelude::OnExit(#state), ((#(#systems),*)))
                });
            }
        }
    }
}
