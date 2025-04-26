use std::ops::{Deref, DerefMut};

use bevy_ecs::resource::Resource;

pub mod event;
pub mod world;

#[derive(Resource)]
pub struct Wrapper<T> {
    inner: T,
}

impl<T> Wrapper<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T> From<T> for Wrapper<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
