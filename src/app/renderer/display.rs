use std::{collections::HashMap, sync::Arc};

use bevy_ecs::{
    resource::Resource,
    system::{Commands, Local, Res, ResMut},
};
use gpu_bytes::AsStd430;
use wgpu::util::DeviceExt;
use wgputil::shader::ShaderSource;

use crate::{
    app::{
        camera::binding::ScreenBinding,
        lookup::CameraResponseBinding,
        renderer::{
            material::MaterialTextures, profiler::RenderProfiler, FrameRecord, RendererViewport,
            SurfaceState,
        },
    },
    util,
};

#[derive(Resource)]
pub struct DisplayBinding {
    sampler: wgpu::Sampler,
    viewport_buffer: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl DisplayBinding {
    pub fn init(
        mut commands: Commands,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        material_textures: Res<MaterialTextures>,
    ) {
        let view = material_textures
            .current_texture
            .create_view(&Default::default());
        let sample_type = wgputil::texture::sample_type(
            &surface_state.gpu.device,
            &material_textures.current_texture,
        )
        .unwrap();

        let sampler = surface_state
            .gpu
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("display_input_sampler"),
                ..Default::default()
            });

        let viewport_buffer =
            surface_state
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("display_viewport_buffer"),
                    contents: renderer_viewport.as_std430().as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &surface_state.gpu.device,
            "display_binding",
            &[
                wgputil::binding::bind_texture(&view, sample_type, wgpu::TextureViewDimension::D2),
                wgputil::binding::bind_sampler(&sampler, wgpu::SamplerBindingType::NonFiltering),
                wgputil::binding::bind_buffer_uniform(&viewport_buffer),
            ],
        );

        commands.insert_resource(Self {
            sampler,
            viewport_buffer,
            bind_group,
            bind_group_layout,
        });
    }

    pub fn on_resize(
        mut display_binding: ResMut<DisplayBinding>,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        material_textures: Res<MaterialTextures>,
    ) {
        surface_state.gpu.queue.write_buffer(
            &display_binding.viewport_buffer,
            0,
            renderer_viewport.as_std430().as_slice(),
        );

        let view = material_textures
            .current_texture
            .create_view(&Default::default());

        display_binding.bind_group = wgputil::binding::create_sequential_with_layout(
            &surface_state.gpu.device,
            "display_input_binding",
            &display_binding.bind_group_layout,
            &[
                wgpu::BindingResource::TextureView(&view),
                wgpu::BindingResource::Sampler(&display_binding.sampler),
                display_binding.viewport_buffer.as_entire_binding(),
            ],
        );
    }
}

#[derive(Resource)]
pub struct DisplayPipelines {
    pipelines: HashMap<Arc<str>, wgpu::RenderPipeline>,
    active_pipeline: Arc<str>,
}

impl DisplayPipelines {
    pub fn init(
        mut commands: Commands,
        surface_state: Res<SurfaceState>,
        screen_binding: Res<ScreenBinding>,
        camera_response_binding: Res<CameraResponseBinding>,
        display_binding: Res<DisplayBinding>,
    ) {
        let gpu = &surface_state.gpu;

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("display_pipeline_layout"),
                bind_group_layouts: &[
                    &screen_binding.bind_group_layout,
                    &camera_response_binding.bind_group_layout,
                    &display_binding.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let mut pipelines = HashMap::new();

        let pipeline_paths = ["final.slang"];
        let active_pipeline = Arc::from(pipeline_paths[0]);

        let vertex_shader_source = ShaderSource::load_spirv(util::shader_path("frame.slang"));
        let vertex_shader_module =
            wgputil::shader::create(&gpu.device, &vertex_shader_source).unwrap();

        for pipeline_path in pipeline_paths {
            let path = util::shader_path(pipeline_path);

            let source = ShaderSource::load_spirv(path);
            let fragment_shader_module = wgputil::shader::create(&gpu.device, &source).unwrap();

            let pipeline = gpu
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(pipeline_path),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &vertex_shader_module,
                        entry_point: Some("vertex"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader_module,
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
                });

            pipelines.insert(Arc::from(pipeline_path), pipeline);
        }

        commands.insert_resource(Self {
            pipelines,
            active_pipeline,
        });
    }

    pub fn get_active_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipelines[&self.active_pipeline]
    }

    #[expect(unused)]
    pub fn set_active_pipeline(&mut self, active_pipeline: &str) {
        self.active_pipeline = Arc::from(active_pipeline);
    }
}

#[expect(clippy::too_many_arguments)]
pub fn draw(
    mut time_query_index: Local<Option<usize>>,

    surface_state: Res<SurfaceState>,
    mut profiler: ResMut<RenderProfiler>,
    mut frame: ResMut<FrameRecord>,

    screen_binding: Res<ScreenBinding>,
    camera_response_binding: Res<CameraResponseBinding>,
    display_binding: Res<DisplayBinding>,

    display_pipelines: Res<DisplayPipelines>,
) {
    if time_query_index.is_none() {
        *time_query_index = Some(profiler.push(&surface_state.gpu, "display_pass"));
    }

    let (_, time_query) = &mut profiler.time_queries[time_query_index.unwrap()];

    let view = frame.surface_texture_view.clone();

    let mut render_pass = frame
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("display_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: Some(time_query.render_timestamp_writes()),
            occlusion_query_set: None,
        });

    render_pass.set_bind_group(0, &screen_binding.bind_group, &[]);
    render_pass.set_bind_group(1, &camera_response_binding.bind_group, &[]);
    render_pass.set_bind_group(2, &display_binding.bind_group, &[]);

    render_pass.set_pipeline(display_pipelines.get_active_pipeline());

    render_pass.draw(0..6, 0..1);
}
