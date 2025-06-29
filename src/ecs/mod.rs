use std::ops::{Deref, DerefMut};

use bevy_ecs::resource::Resource;

mod event;
pub mod events;
pub mod schedule;

#[derive(Resource)]
pub struct ResourceWrapper<T>(T);

impl<T> ResourceWrapper<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for ResourceWrapper<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for ResourceWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ResourceWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
