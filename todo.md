=== Remaining 0.2 Goals ===
[ ] `#[in(state)]` macro
[x] `#[init_state]` include derives

=== Prefabs and Scopes ===
[x] Define prefabs like a macro + structure
    [x] Creation functions
    [x] Marked as global vs local (default: local to state created with)
    [x] When #[plugin] initializes state, add observer to remove locals belonging to the old state
 - Effectively a bundle + some functions and types
[x] Global vs local for state management
 - Functions/events to auto remove non-global objects on scene change
 - Local objects should track what scene they are meant to be apart of
 - Local objects may be able to be apart of multiple scenes
 - Global is defined as nothing but a entity as something that does not belong to a state
 - Local marks an entity as belonging to a state
