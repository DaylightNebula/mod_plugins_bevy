use syn::*;
use quote::quote;

#[derive(Clone, Debug, Default)]
pub(crate) struct InitializationSystems {
    pub events: Vec<Ident>,
    pub registered: Vec<Ident>,
    pub states_def: Vec<Ident>,
    pub states_nodef: Vec<Expr>
}

impl InitializationSystems {
    pub(crate) fn append(&self, app_ext: &mut proc_macro2::TokenStream) {
        for event in &self.events {
            app_ext.extend(quote! {
                .add_event::<#event>()
            });
        }

        for registered in &self.registered {
            app_ext.extend(quote! {
                .register_type::<#registered>()
            });
        }

        for state in &self.states_def {
            app_ext.extend(quote! {
                .init_state::<#state>()
            });
        }

        for state in &self.states_nodef {
            app_ext.extend(quote! {
                .insert_state(#state)
            });
        }
    }
}
