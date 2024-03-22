use syn::Ident;
use quote::quote;

#[derive(Debug, Default, Clone)]
pub(crate) struct BasicSystems {
    startup: Vec<Ident>,
    update: Vec<Ident>
}

impl BasicSystems {
    pub(crate) fn push_startup(&mut self, ident: Ident) {
        self.startup.push(ident)
    }

    pub(crate) fn push_update(&mut self, ident: Ident) {
        self.update.push(ident);
    }

    pub(crate) fn append(&self, app_ext: &mut proc_macro2::TokenStream) {
        if !self.startup.is_empty() {
            let startup = &self.startup;
            app_ext.extend(quote! {
                .add_systems(bevy::prelude::Startup, ((#(#startup),*)))
            });
        }
        if !self.update.is_empty() {
            let update = &self.update;
            app_ext.extend(quote! {
                .add_systems(bevy::prelude::Update, ((#(#update),*)))
            });
        }
    }
}