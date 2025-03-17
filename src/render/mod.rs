use std::sync::Arc;

use bevy_ecs::{entity::Entity, event::Event, system::Resource};
use winit::{dpi::PhysicalSize, window::Window};

pub mod texture;

pub const WGPU_FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE
    .union(wgpu::Features::RG11B10UFLOAT_RENDERABLE)
    .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
    .union(wgpu::Features::PUSH_CONSTANTS)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO)
    .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);

#[derive(Clone)]
pub struct GpuHandle {
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl From<&GpuHandle> for GpuHandle {
    fn from(val: &GpuHandle) -> Self {
        val.clone()
    }
}

#[derive(Resource)]
pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,

    pub gpu_handle: GpuHandle,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: WGPU_FEATURES,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self {
            surface,
            config,
            size,
            window,
            gpu_handle: GpuHandle {
                instance,
                adapter,
                device,
                queue,
            },
        }
    }

    pub fn reconfigure_surface(&self) {
        self.surface
            .configure(&self.gpu_handle.device, &self.config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure_surface();
        }
    }

    pub fn begin_frame(&self) -> Result<FrameData, wgpu::SurfaceError> {
        let encoder =
            self.gpu_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        let surface_texture = self.surface.get_current_texture()?;

        Ok(FrameData {
            encoder,
            surface_texture,
        })
    }

    pub fn finish_frame(&self, frame: FrameData) {
        self.gpu_handle
            .queue
            .submit(std::iter::once(frame.encoder.finish()));
        frame.surface_texture.present();
    }
}

#[derive(Resource)]
pub struct FrameData {
    pub encoder: wgpu::CommandEncoder,
    pub surface_texture: wgpu::SurfaceTexture,
}

#[derive(Event)]
pub struct WindowResizeEvent(pub PhysicalSize<u32>);

#[derive(Event)]
pub struct RenderResourceUpdateEvent(pub Entity);
