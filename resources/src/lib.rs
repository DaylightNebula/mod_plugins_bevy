use std::{fmt::Debug, ops::Deref};
use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct Current<T: Clone + Debug>(T);

/// Allow Current to be dereferenced so it implements all the functions of the wrapped type
impl <T: Clone + Debug> Deref for Current<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <T: Clone + Debug> Current<T> {
    /// Creates a new Current instance to wrap the given type
    pub fn new(input: T) -> Self { Self(input) }

    /// Get a reference to the wrapped object
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get a mutable reference to the wrapped object
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Transform this into the wrapped object
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// A trait to be implemented by structs that need to be able to execute something on the client.
pub trait Executable<O> {
    fn execute(self: Box<Self>, world: &mut World) -> O;
}

#[derive(Component, Default)]
pub struct ScopeGlobal;

#[derive(Component, Default)]
pub struct ScopeLocal<S: States + Default>(pub S);
