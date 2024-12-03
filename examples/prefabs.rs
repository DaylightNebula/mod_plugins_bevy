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
    use mod_plugins_resources::ScopeLocal;

    #[init_state]
    pub enum CubeState { 
        #[default] 
        Exists, 
        DoesNotExist 
    }

    #[prefab(scope local CubeState)]
    pub struct CubePrefab {
        pub mesh: Mesh3d,
        pub material: MeshMaterial3d<StandardMaterial>,
        pub transform: Transform,
        pub global_transform: GlobalTransform,
        pub visibility: Visibility,
        pub inherited_visibility: InheritedVisibility,
        pub view_visibility: ViewVisibility
    }

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

    #[enter(CubeState::Exists)]
    fn make_cube_exist(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>
    ) {
        commands.spawn(CubePrefab {
            mesh: Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            material: MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            scope: ScopeLocal(CubeState::Exists),
            ..Default::default()
        });
    }

    #[event(KeyboardInput)]
    fn keyboard_input(
        state: Res<State<CubeState>>,
        mut next_state: ResMut<NextState<CubeState>>
    ) {
        if keyboard_input.key_code == KeyCode::Space && keyboard_input.state == ButtonState::Released {
            match state.get() {
                CubeState::Exists => next_state.set(CubeState::DoesNotExist),
                CubeState::DoesNotExist => next_state.set(CubeState::Exists)
            }
            println!("Swapping from {:?}", state.get());
        }
    }
}
