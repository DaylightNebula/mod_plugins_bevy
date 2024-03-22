use std::fmt::Debug;
use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct Current<T: Clone + Debug>(pub T);
