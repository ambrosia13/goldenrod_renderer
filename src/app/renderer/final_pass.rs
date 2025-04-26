use bevy_ecs::resource::Resource;
use bevy_ecs::{
    event::EventReader,
    system::{Commands, Res, ResMut},
};
use glam::Vec2;
use gpu_bytes::AsStd140;
use gpu_bytes_derive::AsStd140;

use crate::render::binding::{Binding, BindingEntry};
use crate::render::{
    self,
    buffer::Buffer,
    shader::{Shader, ShaderRecompileEvent, ShaderSource},
    texture::{Texture, TextureType},
    FrameData, GpuHandle, RenderState, WindowResizeEvent,
};

use super::{pathtrace::PathtracePass, profiler::RenderProfiler};

#[derive(AsStd140, Default)]
struct FinalUniform {
    effective_start: Vec2,
    effective_end: Vec2,
}

impl FinalUniform {
    pub fn from_render_state(render_state: &RenderState) -> Self {
        let start = Vec2::new(
            render_state.effective_viewport_start.0 as f32 / render_state.size.width as f32,
            render_state.effective_viewport_start.1 as f32 / render_state.size.height as f32,
        );

        let end = Vec2::new(
            render_state.effective_viewport_end.0 as f32 / render_state.size.width as f32,
            render_state.effective_viewport_end.1 as f32 / render_state.size.height as f32,
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

    uniform_buffer: Buffer,

    texture_binding: Binding,
    camera_response_binding: Binding,
    uniform_binding: Binding,

    vertex_shader: Shader,
    fragment_shader: Shader,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,

    time_query_index: usize,

    gpu_handle: GpuHandle,
}

impl FinalPass {
    fn new(
        render_state: &RenderState,
        input_texture: &Texture,
        profiler: &mut RenderProfiler,
    ) -> Self {
        let gpu_handle = render_state.gpu_handle.clone();

        let sampler = gpu_handle.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("final_input_texture_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let texture_binding =
            Self::create_texture_binding(gpu_handle.clone(), &sampler, input_texture);

        let data = FinalUniform::from_render_state(render_state).as_std140();
        let uniform_buffer = Buffer::with_data(
            gpu_handle.clone(),
            "final_uniform_buffer",
            data.as_slice(),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let uniform_binding = Binding::new(
            gpu_handle.clone(),
            &[uniform_buffer.bind_uniform(0, None, false)],
        );

        let camera_response_binding = Self::create_lut_binding(gpu_handle.clone());

        let pipeline_layout =
            gpu_handle
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("final_pipeline_layout"),
                    bind_group_layouts: &[
                        texture_binding.bind_group_layout(),
                        camera_response_binding.bind_group_layout(),
                        uniform_binding.bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let (vertex_shader, fragment_shader, pipeline) =
            Self::create_shader_and_pipeline(render_state, &pipeline_layout);

        let time_query_index = profiler.push(&gpu_handle, "final");

        Self {
            pipeline,
            sampler,
            texture_binding,
            uniform_buffer,
            uniform_binding,
            time_query_index,
            gpu_handle,
            camera_response_binding,
            vertex_shader,
            fragment_shader,
            pipeline_layout,
        }
    }

    fn create_lut_binding(gpu_handle: GpuHandle) -> Binding {
        let camera_response_red = Texture::load_raw(
            gpu_handle.clone(),
            "assets/textures/camera_response/Kodachrome-64CDRed.bin",
            (1024, 1, 1),
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture1d,
        );

        let camera_response_red_view = camera_response_red.view(0..1, 0..1);

        let camera_response_green = Texture::load_raw(
            gpu_handle.clone(),
            "assets/textures/camera_response/Kodachrome-64CDGreen.bin",
            (1024, 1, 1),
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture1d,
        );

        let camera_response_green_view = camera_response_green.view(0..1, 0..1);

        let camera_response_blue = Texture::load_raw(
            gpu_handle.clone(),
            "assets/textures/camera_response/Kodachrome-64CDBlue.bin",
            (1024, 1, 1),
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture1d,
        );

        let camera_response_blue_view = camera_response_blue.view(0..1, 0..1);

        let camera_response_sampler = gpu_handle.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("final_camera_response_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Binding::new(
            gpu_handle,
            &[
                camera_response_red.bind_view(&camera_response_red_view),
                camera_response_green.bind_view(&camera_response_green_view),
                camera_response_blue.bind_view(&camera_response_blue_view),
                BindingEntry {
                    binding_type: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                    resource: wgpu::BindingResource::Sampler(&camera_response_sampler),
                },
            ],
        )
    }

    fn create_shader_and_pipeline(
        render_state: &RenderState,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> (Shader, Shader, wgpu::RenderPipeline) {
        let vertex_shader = Shader::new(
            render_state.gpu_handle.clone(),
            ShaderSource::load_wgsl("assets/shaders/frame.wgsl"),
        );

        let fragment_shader = Shader::new(
            render_state.gpu_handle.clone(),
            ShaderSource::load_wgsl("assets/shaders/final.wgsl"),
        );

        let pipeline = render_state.gpu_handle.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("final_render_pipeline"),
                layout: Some(pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vertex_shader.module(),
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: fragment_shader.module(),
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: render_state.config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            },
        );

        (vertex_shader, fragment_shader, pipeline)
    }

    fn create_texture_binding(
        gpu_handle: GpuHandle,
        sampler: &wgpu::Sampler,
        input_texture: &Texture,
    ) -> Binding {
        Binding::new(
            gpu_handle,
            &[
                input_texture.bind_view(&input_texture.view(0..1, 0..1)),
                BindingEntry {
                    binding_type: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::NonFiltering,
                    ),
                    count: None,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        )
    }

    fn draw(&self, frame: &mut FrameData, profiler: &mut RenderProfiler) {
        let (_, time_query) = &mut profiler.time_queries[self.time_query_index];

        time_query.write_start_timestamp(&mut frame.encoder);

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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_bind_group(0, self.texture_binding.bind_group(), &[]);
        render_pass.set_bind_group(1, self.camera_response_binding.bind_group(), &[]);
        render_pass.set_bind_group(2, self.uniform_binding.bind_group(), &[]);
        render_pass.set_pipeline(&self.pipeline);

        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        time_query.write_end_timestamp(&mut frame.encoder);
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        pathtrace: Res<PathtracePass>,
        mut profiler: ResMut<RenderProfiler>,
    ) {
        let display = Self::new(&render_state, &pathtrace.color_texture, &mut profiler);
        commands.insert_resource(display);
    }

    pub fn update(
        mut display: ResMut<FinalPass>,
        pathtrace: Res<PathtracePass>,
        mut frame: ResMut<FrameData>,
        render_state: Res<RenderState>,
        mut profiler: ResMut<RenderProfiler>,

        mut resize_events: EventReader<WindowResizeEvent>,
        mut shader_recompile_events: EventReader<ShaderRecompileEvent>,
    ) {
        if resize_events.read().count() > 0 {
            display.texture_binding = Self::create_texture_binding(
                display.gpu_handle.clone(),
                &display.sampler,
                &pathtrace.color_texture,
            )
        }

        if shader_recompile_events.read().count() > 0 {
            let (vertex, fragment, pipeline) =
                Self::create_shader_and_pipeline(&render_state, &display.pipeline_layout);

            display.vertex_shader = vertex;
            display.fragment_shader = fragment;
            display.pipeline = pipeline;
        }

        let data = FinalUniform::from_render_state(&render_state);
        display.uniform_buffer.write(data.as_std140().as_slice(), 0);

        display.draw(&mut frame, &mut profiler);
    }
}
