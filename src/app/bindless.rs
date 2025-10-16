use crate::app::renderer::SurfaceState;
use bevy_ecs::change_detection::{DetectChanges, DetectChangesMut};
use bevy_ecs::prelude::{Commands, Res, Resource};
use bevy_ecs::system::ResMut;
use glam::UVec2;
use gpu_layout::AsGpuBytes;
use std::num::NonZeroU32;

#[derive(AsGpuBytes, Copy, Clone, Debug)]
pub struct DescriptorHandle {
    indices: UVec2,
}

pub const BINDLESS_SAMPLER_COUNT: u32 = 100;
pub const BINDLESS_TEXTURE_COUNT: u32 = 100;
pub const BINDLESS_BUFFER_COUNT: u32 = 100;

#[derive(Resource)]
pub struct BindlessHeap {
    samplers: Vec<wgpu::Sampler>,
    texture_views: Vec<wgpu::TextureView>,
    buffers: Vec<wgpu::Buffer>,

    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

    device: wgpu::Device,
}

impl BindlessHeap {
    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        let dummy_sampler = surface_state.gpu.device.create_sampler(&Default::default());
        let dummy_texture_view = surface_state
            .gpu
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("dummy_texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let dummy_buffer = surface_state
            .gpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("dummy_buffer"),
                size: 16,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

        let bind_group_layout =
            surface_state
                .gpu
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bindless_heap_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: NonZeroU32::new(BINDLESS_SAMPLER_COUNT),
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: NonZeroU32::new(BINDLESS_TEXTURE_COUNT),
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: NonZeroU32::new(BINDLESS_BUFFER_COUNT),
                        },
                    ],
                });

        let bind_group = surface_state
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bindless_heap"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::SamplerArray(&[&dummy_sampler]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureViewArray(&[&dummy_texture_view]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::BufferArray(&[
                            dummy_buffer.as_entire_buffer_binding()
                        ]),
                    },
                ],
            });

        commands.insert_resource(Self {
            samplers: vec![dummy_sampler],
            texture_views: vec![dummy_texture_view],
            buffers: vec![dummy_buffer],
            bind_group,
            bind_group_layout,
            device: surface_state.gpu.device.clone(),
        });
    }

    pub fn insert_sampler(&mut self, sampler: &wgpu::Sampler) -> DescriptorHandle {
        self.samplers.push(sampler.clone());
        self.update_bind_group();

        DescriptorHandle {
            indices: UVec2::splat(self.samplers.len() as u32 - 1),
        }
    }

    pub fn insert_texture_view(&mut self, texture_view: &wgpu::TextureView) -> DescriptorHandle {
        self.texture_views.push(texture_view.clone());
        self.update_bind_group();

        DescriptorHandle {
            indices: UVec2::splat(self.texture_views.len() as u32 - 1),
        }
    }

    pub fn insert_buffer(&mut self, buffer: &wgpu::Buffer) -> DescriptorHandle {
        self.buffers.push(buffer.clone());
        self.update_bind_group();

        DescriptorHandle {
            indices: UVec2::splat(self.buffers.len() as u32 - 1),
        }
    }

    fn update_bind_group(&mut self) {
        log::info!(
            "updating bindless heap bind group, s: {}, t: {}, b: {}",
            // subtract 1 because of the dummy objects
            self.samplers.len() - 1,
            self.texture_views.len() - 1,
            self.buffers.len() - 1,
        );

        self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bindless_heap"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::SamplerArray(
                        &self.samplers.iter().collect::<Vec<_>>(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &self.texture_views.iter().collect::<Vec<_>>(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::BufferArray(
                        &self
                            .buffers
                            .iter()
                            .map(|b| b.as_entire_buffer_binding())
                            .collect::<Vec<_>>(),
                    ),
                },
            ],
        })
    }
}

pub struct LookupTable {
    data: Vec<f32>,
    min: f32,
    max: f32,
    default_value: f32,
}
