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
    #[system(startup)]
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>
    ) {
        println!("Startup");

        // circular base
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Visibility::default()
        ));

        // cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Visibility::default()
        ));
        
        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
        
        // camera
        commands.spawn((
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            Camera3d::default()
        ));
    }

    #[system(update)]
    fn update0() { println!("Update 0") }
    
    #[system(update)]
    #[before(update0)]
    fn update1() { println!("Update 1") }
}