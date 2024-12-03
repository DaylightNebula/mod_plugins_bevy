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
    use bevy::input::{keyboard::KeyboardInput, ButtonState};

    #[init_state]
    // #[derive(States, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    enum TestState {
        #[default]
        Red,
        Blue
    }    

    #[derive(Component)]
    struct Red;
    
    #[derive(Component)]
    struct Blue;

    #[startup]
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // circular base
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
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

    #[build]
    fn build_test(&self, app: &mut App) {
        println!("App {app:#?}");
    }

    #[event(KeyboardInput)]
    fn keyboard_input(
        state: Res<State<TestState>>,
        mut next_state: ResMut<NextState<TestState>>
    ) {
        if keyboard_input.key_code == KeyCode::Space && keyboard_input.state == ButtonState::Released {
            match state.get() {
                TestState::Red => next_state.set(TestState::Blue),
                TestState::Blue => next_state.set(TestState::Red)
            }
            println!("Swapping from {:?}", state.get());
        }
    }

    #[enter(TestState::Red)]
    fn start_red(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 124))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Visibility::default(),
            Red
        ));
    }

    #[exit(TestState::Red)]
    fn end_red(
        mut commands: Commands,
        query: Query<Entity, With<Red>>
    ) {
        query.iter().for_each(|entity| {
            commands.entity(entity).despawn_recursive();
        });
    }

    #[enter(TestState::Blue)]
    fn start_blue(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Visibility::default(),
            Blue
        ));
    }

    #[exit(TestState::Blue)]
    fn end_blue(
        mut commands: Commands,
        query: Query<Entity, With<Red>>
    ) {
        query.iter().for_each(|entity| {
            commands.entity(entity).despawn_recursive();
        });
    }
}
