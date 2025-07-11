use bevy_ecs::{
    resource::Resource,
    system::{Commands, Res, ResMut},
};
use glam::{Mat4, Vec3};
use gpu_bytes::AsStd140;
use gpu_bytes_derive::{AsStd140, AsStd430};
use wgpu::util::DeviceExt;

use crate::app::renderer::{RendererViewport, SurfaceState};

use super::Camera;

#[derive(Resource)]
pub struct ScreenBinding {
    pub camera_uniform: CameraUniform,
    pub view_uniform: ViewUniform,

    pub camera_buffer: wgpu::Buffer,
    pub view_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ScreenBinding {
    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        // use default values when creating
        let camera_uniform = CameraUniform::default();
        let view_uniform = ViewUniform::default();

        let camera_buffer =
            surface_state
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("camera_buffer"),
                    contents: camera_uniform.as_std140().as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let view_buffer =
            surface_state
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("view_buffer"),
                    contents: view_uniform.as_std140().as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &surface_state.gpu.device,
            "screen_binding",
            &[
                wgputil::binding::bind_buffer_uniform(&camera_buffer),
                wgputil::binding::bind_buffer_uniform(&view_buffer),
            ],
        );

        let screen_binding = Self {
            camera_uniform,
            view_uniform,
            camera_buffer,
            view_buffer,
            bind_group_layout,
            bind_group,
        };

        commands.insert_resource(screen_binding);
    }

    pub fn update(
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        mut screen_binding: ResMut<ScreenBinding>,
        camera: Res<Camera>,
    ) {
        screen_binding.camera_uniform.update_from(&camera);
        wgputil::buffer::write_slice(
            &surface_state.gpu.queue,
            &screen_binding.camera_buffer,
            screen_binding.camera_uniform.as_std140().as_slice(),
            0,
        );

        screen_binding
            .view_uniform
            .update_from(&renderer_viewport, &surface_state);
        wgputil::buffer::write_slice(
            &surface_state.gpu.queue,
            &screen_binding.view_buffer,
            screen_binding.view_uniform.as_std140().as_slice(),
            0,
        );
    }
}

#[derive(AsStd140, AsStd430, Default)]
pub struct CameraUniform {
    view_projection_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,

    inverse_view_projection_matrix: Mat4,
    inverse_view_matrix: Mat4,
    inverse_projection_matrix: Mat4,

    previous_view_projection_matrix: Mat4,
    previous_view_matrix: Mat4,
    previous_projection_matrix: Mat4,

    position: Vec3,
    previous_position: Vec3,

    view: Vec3,
    previous_view: Vec3,

    right: Vec3,
    up: Vec3,
}

impl CameraUniform {
    fn update_from(&mut self, camera: &Camera) {
        self.previous_projection_matrix = self.view_projection_matrix;
        self.previous_view_matrix = self.view_matrix;
        self.previous_projection_matrix = self.projection_matrix;

        self.view_matrix = camera.view_matrix();
        self.projection_matrix = camera.projection_matrix();
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;

        self.inverse_view_matrix = self.view_matrix.inverse();
        self.inverse_projection_matrix = self.projection_matrix.inverse();
        self.inverse_view_projection_matrix = self.view_projection_matrix.inverse();

        self.previous_position = self.position;
        self.position = camera.position;

        self.previous_view = self.view;
        self.view = camera.forward();

        self.right = camera.right();
        self.up = camera.up();
    }
}

#[derive(AsStd140, AsStd430, Default)]
pub struct ViewUniform {
    renderer_viewport_width: u32,
    renderer_viewport_height: u32,
    window_width: u32,
    window_height: u32,
    aspect_ratio: f32,
    frame_count: u32,
}

impl ViewUniform {
    fn update_from(&mut self, renderer_viewport: &RendererViewport, surface_state: &SurfaceState) {
        self.renderer_viewport_width = renderer_viewport.get_width();
        self.renderer_viewport_height = renderer_viewport.get_height();
        self.window_width = surface_state.viewport_size.width;
        self.window_height = surface_state.viewport_size.height;
        self.aspect_ratio =
            self.renderer_viewport_width as f32 / self.renderer_viewport_height as f32;
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}
