use bevy::prelude::*;
use mod_plugins::macros::*;

#[derive(States, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum TestState {
    #[default]
    Red,
    Blue
}

#[derive(Component)]
struct Red;

#[derive(Component)]
struct Blue;

fn main() {
    App::new()
        .init_state::<TestState>()
        .add_plugins((DefaultPlugins, TestPlugin))
        .run();
}

#[plugin]
mod test_plugin {
    use bevy::input::{keyboard::KeyboardInput, ButtonState};

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

    #[event(KeyboardInput)]
    fn keyboard_input(
        state: Res<State<TestState>>,
        mut next_state: ResMut<NextState<TestState>>
    ) {
        if current.0.key_code == KeyCode::Space && current.0.state == ButtonState::Released {
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
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::rgb_u8(255, 144, 124)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
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
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::rgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
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
