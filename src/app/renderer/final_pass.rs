use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Res, ResMut};
use glam::Vec2;
use gpu_bytes::AsStd140;
use gpu_bytes_derive::AsStd140;
use wgpu::util::DeviceExt;
use wgputil::GpuHandle;

use crate::app::lookup::CameraResponseBinding;
use crate::app::renderer::{FrameRecord, RendererViewport, SurfaceState};
use crate::util;

use super::{pathtrace::PathtracePass, profiler::RenderProfiler};

#[derive(AsStd140, Default)]
struct FinalUniform {
    effective_start: Vec2,
    effective_end: Vec2,
}

impl FinalUniform {
    pub fn from_renderer_viewport(
        surface_state: &SurfaceState,
        renderer_viewport: &RendererViewport,
    ) -> Self {
        let start = Vec2::new(
            renderer_viewport.start.x as f32 / surface_state.viewport_size.width as f32,
            renderer_viewport.start.y as f32 / surface_state.viewport_size.height as f32,
        );

        let end = Vec2::new(
            renderer_viewport.end.x as f32 / surface_state.viewport_size.width as f32,
            renderer_viewport.end.y as f32 / surface_state.viewport_size.height as f32,
        );

        Self {
            effective_start: start,
            effective_end: end,
        }
    }
}

#[derive(Resource)]
pub struct FinalPass {
    sampler: wgpu::Sampler,

    uniform_buffer: wgpu::Buffer,

    texture_bind_group_layout: wgpu::BindGroupLayout,

    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,

    vertex_shader_source: wgputil::shader::ShaderSource,
    fragment_shader_source: wgputil::shader::ShaderSource,
    vertex_shader: wgpu::ShaderModule,
    fragment_shader: wgpu::ShaderModule,

    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,

    time_query_index: usize,
}

impl FinalPass {
    fn new(
        surface_state: &SurfaceState,
        renderer_viewport: &RendererViewport,
        camera_response_binding: &CameraResponseBinding,
        input_texture: &wgpu::Texture,
        profiler: &mut RenderProfiler,
    ) -> Self {
        let sampler = surface_state
            .gpu
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("final_input_texture_sampler"),
                ..Default::default()
            });

        let (texture_bind_group_layout, texture_bind_group) =
            Self::create_texture_binding(&surface_state.gpu.device, &sampler, input_texture);

        let data =
            FinalUniform::from_renderer_viewport(surface_state, renderer_viewport).as_std140();
        let uniform_buffer =
            surface_state
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("final_uniform_buffer"),
                    contents: data.as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let (uniform_bind_group_layout, uniform_bind_group) =
            wgputil::binding::create_sequential_linked(
                &surface_state.gpu.device,
                "uniform_binding",
                &[wgputil::binding::bind_buffer_uniform(&uniform_buffer)],
            );

        let pipeline_layout =
            surface_state
                .gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("final_pipeline_layout"),
                    bind_group_layouts: &[
                        &camera_response_binding.bind_group_layout,
                        &texture_bind_group_layout,
                        &uniform_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let (vertex_shader_source, fragment_shader_source, vertex_shader, fragment_shader) =
            Self::create_shaders(&surface_state.gpu.device);

        let pipeline = Self::create_pipeline(
            surface_state,
            &pipeline_layout,
            &vertex_shader,
            &fragment_shader,
        );

        let time_query_index = profiler.push(&surface_state.gpu, "final");

        Self {
            pipeline,
            sampler,
            texture_bind_group_layout,
            texture_bind_group,
            uniform_buffer,
            uniform_bind_group,
            time_query_index,
            vertex_shader_source,
            fragment_shader_source,
            vertex_shader,
            fragment_shader,
            pipeline_layout,
        }
    }

    fn create_shaders(
        device: &wgpu::Device,
    ) -> (
        wgputil::shader::ShaderSource,
        wgputil::shader::ShaderSource,
        wgpu::ShaderModule,
        wgpu::ShaderModule,
    ) {
        let mut vertex_shader_source =
            wgputil::shader::ShaderSource::load_spirv(util::shader_path("frame.slang"));

        let mut fragment_shader_source =
            wgputil::shader::ShaderSource::load_spirv(util::shader_path("final.slang"));

        let (vertex_shader, _error) =
            wgputil::shader::create_or_fallback(device, &mut vertex_shader_source);

        let (fragment_shader, _error) =
            wgputil::shader::create_or_fallback(device, &mut fragment_shader_source);

        (
            vertex_shader_source,
            fragment_shader_source,
            vertex_shader,
            fragment_shader,
        )
    }

    fn create_pipeline(
        surface_state: &SurfaceState,
        pipeline_layout: &wgpu::PipelineLayout,
        vertex: &wgpu::ShaderModule,
        fragment: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        surface_state
            .gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("final_render_pipeline"),
                layout: Some(pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vertex,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: fragment,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_state.config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            })
    }

    fn create_texture_binding(
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        input_texture: &wgpu::Texture,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        wgputil::binding::create_sequential_linked(
            device,
            "final_texture_binding",
            &[
                wgputil::binding::bind_texture(
                    &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    wgputil::texture::sample_type(device, input_texture).unwrap(),
                    wgpu::TextureViewDimension::D2,
                ),
                wgputil::binding::bind_sampler(sampler, wgpu::SamplerBindingType::NonFiltering),
            ],
        )
    }

    fn draw(
        &self,
        frame: &mut FrameRecord,
        profiler: &mut RenderProfiler,
        camera_response_binding: &CameraResponseBinding,
    ) {
        let (_, time_query) = &mut profiler.time_queries[self.time_query_index];

        let view = frame
            .surface_texture
            .texture
            .create_view(&Default::default());

        let mut render_pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("final_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Some(time_query.render_timestamp_writes()),
                occlusion_query_set: None,
            });

        render_pass.set_bind_group(0, &camera_response_binding.bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
        render_pass.set_pipeline(&self.pipeline);

        render_pass.draw(0..6, 0..1);
        drop(render_pass);
    }

    pub fn init(
        mut commands: Commands,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        camera_response_binding: Res<CameraResponseBinding>,
        pathtrace: Res<PathtracePass>,
        mut profiler: ResMut<RenderProfiler>,
    ) {
        let display = Self::new(
            &surface_state,
            &renderer_viewport,
            &camera_response_binding,
            &pathtrace.color_texture,
            &mut profiler,
        );
        commands.insert_resource(display);
    }

    pub fn update(
        final_pass: Res<FinalPass>,
        mut frame: ResMut<FrameRecord>,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        mut profiler: ResMut<RenderProfiler>,
        camera_response_binding: Res<CameraResponseBinding>,
    ) {
        let data = FinalUniform::from_renderer_viewport(&surface_state, &renderer_viewport);
        wgputil::buffer::write_slice(
            &surface_state.gpu.queue,
            &final_pass.uniform_buffer,
            data.as_std140().as_slice(),
            0,
        );

        final_pass.draw(&mut frame, &mut profiler, &camera_response_binding);
    }

    pub fn on_resize(
        mut final_pass: ResMut<FinalPass>,
        surface_state: Res<SurfaceState>,
        pathtrace: Res<PathtracePass>,
    ) {
        final_pass.texture_bind_group = wgputil::binding::create_sequential_with_layout(
            &surface_state.gpu.device,
            "final_texture_binding",
            &final_pass.texture_bind_group_layout,
            &[
                wgpu::BindingResource::TextureView(
                    &pathtrace.color_texture.create_view(&Default::default()),
                ),
                wgpu::BindingResource::Sampler(&final_pass.sampler),
            ],
        );
    }
}
