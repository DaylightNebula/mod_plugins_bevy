use bevy::{prelude::*, render::{settings::{Backends, RenderCreation, WgpuSettings}, RenderPlugin}};
use mod_plugins::macros::*;

fn main() {
    let mut backends = Backends::all();
    backends.remove(Backends::DX12);

    App::new()
        // set render backends
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(
                WgpuSettings {
                    backends: Some(backends),
                    ..Default::default()
                }
            ),
            ..Default::default()
        }))

        // add test and run
        .add_plugins(TestPlugin).run();
}

#[plugin]
mod test_plugin {
    
    #[init_resource]
    #[derive(Resource)]
    struct ResourceA(pub i32);
    
    impl Default for ResourceA {
        fn default() -> Self {
            Self(1)
        }
    }

    #[derive(Resource)]
    struct ResourceB(pub i32);
    
    #[derive(Resource)]
    struct ResourceC(pub i32);

    #[resource_factory]
    fn create_b() -> ResourceB { ResourceB(2) }

    #[resource_system]
    fn setup(
        mut commands: Commands,
        a: Res<ResourceA>,
        b: Res<ResourceB>
    ) -> ResourceC {
        // camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });

        ResourceC(a.0 + b.0)
    }

    #[update]
    fn update(
        c: Res<ResourceC>
    ) {
        println!("Found C {}", c.0);
        assert_eq!(c.0, 3)
    }
}
