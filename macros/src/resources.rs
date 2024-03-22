use quote::quote;
use syn::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceSystems {
    defaults: Vec<Ident>,           // Resources that need to be initialized via their default
    factories: Vec<Ident>,          // Factory functions to create resources
    systems: Vec<Ident>             // Systems to call
}

impl ResourceSystems {
    pub(crate) fn push_default(&mut self, name: Ident) {
        self.defaults.push(name);
    }

    pub(crate) fn push_factory(&mut self, factory: Ident) {
        self.factories.push(factory);
    }

    pub(crate) fn push_system(&mut self, system: Ident) {
        self.systems.push(system);
    }

    pub(crate) fn append(
        &mut self, 
        output: &mut proc_macro2::TokenStream,
        app_ext: &mut proc_macro2::TokenStream
    ) {
        // add defaults
        if !self.defaults.is_empty() {
            for default in &self.defaults {
                app_ext.extend(quote! {
                    .init_resource::<#default>()
                });
            }
        }

        // add factories
        if !self.factories.is_empty() {
            for factory in &self.factories {
                app_ext.extend(quote! {
                    .insert_resource(#factory())
                });
            }
        }

        // add create systems
        if !self.systems.is_empty() {
            let mut systems = proc_macro2::TokenStream::new();
            for system in &self.systems {
                systems.extend(quote! {
                    {
                        let mut system = bevy::prelude::IntoSystem::into_system(#system);

                        system.initialize(world);
                        let resource = system.run((), world);
                        system.apply_deferred(world);

                        world.insert_resource(resource);
                    }
                });
            }

            output.extend(quote! {
                fn _insert_resources(world: &mut bevy::prelude::World) {
                    #systems
                }
            });

            app_ext.extend(quote! {
                .add_systems(bevy::prelude::Startup, _insert_resources)
            });
        }
    }
}