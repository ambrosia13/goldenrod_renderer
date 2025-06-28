use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bevy_ecs::{event::Event, resource::Resource};
use winit::{dpi::PhysicalSize, window::Window};

pub const WGPU_FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE
    .union(wgpu::Features::RG11B10UFLOAT_RENDERABLE)
    .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
    .union(wgpu::Features::PUSH_CONSTANTS)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO)
    .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
    .union(wgpu::Features::TIMESTAMP_QUERY)
    .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
    .union(wgpu::Features::VERTEX_WRITABLE_STORAGE)
    .union(wgpu::Features::SPIRV_SHADER_PASSTHROUGH);

#[derive(Resource)]
pub struct SurfaceState {
    pub inner: wgputil::SurfaceState,

    pub effective_viewport_start: (u32, u32), // effective size considering panel UI
    pub effective_viewport_end: (u32, u32),   // effective size considering panel UI
}

impl Deref for SurfaceState {
    type Target = wgputil::SurfaceState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SurfaceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SurfaceState {
    pub async fn new(window: Arc<Window>) -> Self {
        let inner = wgputil::SurfaceState::new(
            window,
            WGPU_FEATURES,
            wgpu::Limits {
                max_push_constant_size: 128,
                ..Default::default()
            },
        )
        .await;

        Self {
            effective_viewport_end: (inner.viewport_size.width, inner.viewport_size.height),
            effective_viewport_start: (0, 0),
            inner,
        }
    }

    pub fn get_effective_width(&self) -> u32 {
        self.effective_viewport_end.0 - self.effective_viewport_start.0
    }

    pub fn get_effective_height(&self) -> u32 {
        self.effective_viewport_end.1 - self.effective_viewport_start.1
    }

    pub fn get_effective_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.get_effective_width(),
            height: self.get_effective_height(),
        }
    }
}

#[derive(Resource)]
pub struct FrameRecord(pub wgputil::FrameRecord);

impl Deref for FrameRecord {
    type Target = wgputil::FrameRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrameRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Event)]
pub struct WindowResizeEvent;
