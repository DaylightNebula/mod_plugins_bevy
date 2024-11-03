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

#[derive(Clone, Debug)]
pub struct Cube(pub Color);

#[executable(Cube)]
fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let color = current.into_inner().clone().into_inner().0;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(color),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

#[plugin]
mod test_plugin {
    use mod_plugins_resources::Executable;

    #[startup]
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // circular base
        commands.spawn(PbrBundle {
            mesh: meshes.add(Circle::new(4.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..default()
        });
        
        // light
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        });
        
        // camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });
    }

    #[startup]
    fn use_cube(world: &mut World) {
        let cube = Box::new(Cube(Color::srgb_u8(124, 255, 124)));
        cube.execute(world);
    }

    #[update]
    #[query(&mut Transform, With<Visibility>)]
    #[on(query1)]
    fn test_cube() {
        // println!("found {query1:?}");
    }

    #[added(Transform)]
    fn test_transform_added(query: Query<&Transform>) {
        let transform = query.get(trigger.entity());
        println!("Added {transform:?}");
    }
}
