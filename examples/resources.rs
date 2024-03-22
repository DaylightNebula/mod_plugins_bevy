use bevy::prelude::*;
use mod_plugins::macros::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TestPlugin))
        .run();
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
