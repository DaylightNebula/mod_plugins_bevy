=== Prefabs and Scopes ===
[ ] Define prefabs like a macro + structure
    [ ] Creation functions
    [ ] Marked as global vs local (default: local to state created with)
    [ ] When #[plugin] initializes state, add observer to remove locals belonging to the old state
    [ ] Match prefab functions + types for queries
 - Effectively a bundle + some functions and types
[ ] Global vs local for scene management
 - Functions/events to auto remove non-global objects on scene change
 - Local objects should track what scene they are meant to be apart of
 - Local objects may be able to be apart of multiple scenes
 - Global is defined as nothing but a entity as something that does not belong to a state
 - Local marks an entity as belonging to a state
[ ] `#[matches(prefab)]` in plugin macro that maps to a special query
