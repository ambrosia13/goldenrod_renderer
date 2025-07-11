use bevy_ecs::event::Event;
use derive_deref::{Deref, DerefMut};

#[derive(Event)]
pub struct MenuResizeEvent;

#[derive(Event, Deref, DerefMut)]
pub struct MouseMotion(pub glam::DVec2);

#[derive(Event, Deref, DerefMut)]
pub struct KeyEvent(pub winit::event::KeyEvent);

#[derive(Event)]
pub struct MouseInput {
    pub state: winit::event::ElementState,
    pub button: winit::event::MouseButton,
}
