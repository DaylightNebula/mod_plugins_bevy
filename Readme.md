# File Plugins for Bevy
This is an experiment/project in removing more boilerplate from the Bevy game engine plugins system.  While the plugins system does not have much boiler plate, this is an experiment in reducing it even more using an attribute macro.  This is achieved by applying the `plugin` attribute macro to a `mod` module in your code, the `mod` block is effectively removed and a plugin in generated.  Here is an example of this:

```rust
// Create a plugin named `TestPlugin` (snake case names of the mod is turned into cammel case).
#[plugin]
mod test_plugin {

    // This system will be run in the `Startup` schedule.
    #[startup]
    fn setup() { some system ... }
}
```

## Marker Attributes
Systems, functions, structs and enums can be marked by attributes (like `#[startup]` above) that apply various plugin related functionality.  

### Startup and Update Systems
The `Startup` and `Update` schedules are crucial to Bevy plugins.  So to add systems to those schedules simply mark them with `#[startup]` or `#[update]`.

```rust
// Create a plugin named `TestPlugin`.
#[plugin]
mod test_plugin {

    // This system will be run in the `Startup` schedule.
    #[startup]
    fn setup() { some system ... }

    // This system will be run in the `Update` schedule.
    #[update]
    fn update() { some system ... }
}
```

### Run on Enter/Exit State Systems
Being able to run systems on enter and exit from states in central to Bevy.  Usually this can be done with the `OnEnter(<some state>)` and `OnExit(<some state>)` schedules.  This can be done with mod plugins by applying the following to your system `#[enter(<some state>)]` or `#[exit(<some state>)]` just like you could with `#[startup]` or `#[update]` above.

### Run on Event Systems
When using Bevy, it is common to run something when an event is fired.  This marker attribute can be used to run a system when an event is fired.  This attribute is `#[event(<some event type>)]`.  These systems will automatically read the `Res<Current<the same event type>>` resource that is temporarily added to the world to run these systems so that these systems have access to the event that was fired.

```rust
// Create a plugin named `TestPlugin`.
#[plugin]
mod test_plugin {
    // This system is run every time a `KeyboardInput` is fired.
    #[event(KeyboardInput)]
    fn keyboard_input() {
        println!("Input {current:?}");
    }
}
```

### Resource Initialization Factories and Systems
Bevy plugins are also responsible for initializing resources, while you can initialize resources by their default implementation which we will discuss later, the plugin macro gives two two marker attributes that can be used to initialize those resources.

The first is `#[resource_factory]` which allows you to mark a function that takes *no* inputs and return a created resource.  This function is ran when the plugin is built, and adds the returned resource to the app.  Here's an example:

```rust
#[plugin]
mod test_plugin {
    // When the plugin is built, this creates and adds `ResourceB` to the app.
    #[resource_factory]
    fn create_b() -> ResourceB { ResourceB(2) }
}
```

The second is `#[resource_system]` which allows you to mark a system that returns a resource.  That system is run on startup and the returned resource is added to the world.  Here's an example:

```rust
#[plugin]
mod test_plugin {
    // On `Startup`, this creates and adds `ResourceC` to the app.
    #[resource_system]
    fn create_resource(
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
}
```

### Build Functions
Sometimes, however, it may be necessary for you to access the app when the plugin is built like you would with a normal Bevy plugin.  You can do this by marking a function that returns nothing and takes in a mutable reference to `App` marked with `#[build]`.  Here's an example of how you can do this:

```rust
#[plugin]
mod test_plugin {
    // This is run when the plugin is built.
    #[build]
    fn on_build(app: &mut App) {
        ... do whatever with the app
    }
}
```

### Auto-Init Events
Plugins need to be able to add their events to the `App`.  You can do this by adding the `#[init_event]` marker attribute to the event you created in a `#[plugin]` mod and the event will be added to he `App` by the plugin when it is built.

```rust
#[plugin]
mod test_plugin {
    #[init_event]
    #[derive(Event)]
    pub struct SomeEvent(pub i32);
}
```

### Auto-Init Resources
Resources have the option of being initialized by their `Default` implementation that they may have.  To do this, just mark the resource that is created in a mod marked with `#[plugin]` with `#[init_resource]`.  The resource will then be added to the `App` by their `Default` implementation when the plugin is built.

```rust
#[plugin]
mod test_plugin {
    #[init_resource]
    #[derive(Resource)]
    pub struct SomeResource {
        name: String,
        num: i32
    }
}
```

### Auto-Init States
Plugins are also responsible for adding `State`s to the `App` when they are built.  You can do this by two ways, either by the `State`s `Default` implementation or by specifying a starting `State`.  You can do this by marking the `State` created in a `#[plugin]` mod with `#[init_state]` if the `State` has a `Default` implementation.  Otherwise, you will need to specify the starting `State` by marking the `State` with `#[init_state(State::Kind)]`.

```rust
#[plugin]
mod test_plugin {
    #[init_state]
    #[derive(States, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ExampleState {
        #[default]
        StateA,
        StateB
    }

    ----------- OR -----------

    #[init_state(ExampleState::StateA)]
    #[derive(States, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ExampleState {
        StateA,
        StateB
    }
}
```

### Register Types
For reflection, you will need to register types with the `App`.  Usually you would do this by calling the `register_type` function on the `App` when the plugin is built.  You can do this in `#[plugin]` mod by marking a struct to register with the `#[register]` attribute marker.

```rust
#[plugin]
mod test_plugin {
    #[register]
    pub struct SomeType(pub u32);
}
```

## Executables
Something useful could be adding systems to data like structs or enums.  This would be useful for generic types where different systems may have to be run depending on the actual type.  For example, a server sends an "action" to the client, and the client runs the system to apply that "action".  Here's how you could make a struct an "executable" struct:

```rust
pub struct ExecutableData {
    name: String,
    data: i32
}

#[executable(ExecutableData)]
fn execute_data() { ... some system }
```

To execute the above system, I could take an instance of `ExecutableData` and call its `execute` function that has been implemented for `ExecutableData` which takes a mutable reference to `World`.  This runs the system for the given world.

```rust
fn use_executable(world: &mut World, data: ExecutableData) {
    data.execute(world);
}
```
