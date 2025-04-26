use std::num::NonZero;

use bevy_ecs::{entity::Entity, world::World};

use super::GpuHandle;

pub fn create_linked(
    device: &wgpu::Device,
    label: &str,
    layout_entries: &[wgpu::BindGroupLayoutEntry],
    entries: &[wgpu::BindGroupEntry],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: layout_entries,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &layout,
        entries,
    });

    (layout, bind_group)
}

pub struct BindingEntry<'a> {
    pub binding_type: wgpu::BindingType,
    pub count: Option<usize>,

    pub resource: wgpu::BindingResource<'a>,
}

impl<'a> BindingEntry<'a> {
    pub fn build(&self, index: usize) -> (wgpu::BindGroupLayoutEntry, wgpu::BindGroupEntry<'a>) {
        (
            wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: wgpu::ShaderStages::all(), // only a thing on directx, skip specifying
                ty: self.binding_type,
                count: self.count.map(|c| c as u32).and_then(NonZero::new),
            },
            wgpu::BindGroupEntry {
                binding: index as u32,
                resource: self.resource.clone(),
            },
        )
    }
}

pub struct Binding {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Binding {
    pub fn new(gpu_handle: impl Into<GpuHandle>, entries: &[BindingEntry]) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let built_entries = entries.iter().enumerate().map(|(i, e)| e.build(i));

        let bind_group_layout_entries: Vec<_> = built_entries.clone().map(|(e, _)| e).collect();
        let bind_group_entries: Vec<_> = built_entries.clone().map(|(_, e)| e).collect();

        let bind_group_layout =
            gpu_handle
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &bind_group_layout_entries,
                });

        let bind_group = gpu_handle
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &bind_group_entries,
            });

        Self {
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}
