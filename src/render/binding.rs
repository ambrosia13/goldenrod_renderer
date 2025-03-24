use std::num::NonZero;

use bevy_ecs::{entity::Entity, world::World};

pub struct BindingEntry<'a> {
    pub binding_type: wgpu::BindingType,
    pub count: Option<usize>,

    pub resource: wgpu::BindingResource<'a>,
}

pub struct Binding {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Binding {}
