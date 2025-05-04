use bevy_ecs::resource::Resource;
use bevy_ecs::{
    event::EventReader,
    system::{Commands, Res, ResMut},
};
use glam::UVec3;

use crate::app::object::binding::ObjectBinding;
use crate::{
    app::camera::binding::ScreenBinding,
    render::{
        shader::{Shader, ShaderRecompileEvent, ShaderSource},
        texture::{Texture, TextureType},
        FrameData, GpuHandle, RenderState, WindowResizeEvent,
    },
};

use super::profiler::RenderProfiler;

#[derive(Resource)]
pub struct PathtracePass {
    pub color_texture: Texture,
    pub previous_color_texture: Texture,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,

    lut_bind_group: wgpu::BindGroup,

    shader: Shader,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,

    time_query_index: usize,

    _gpu_handle: GpuHandle,
}

impl PathtracePass {
    fn new(
        render_state: &RenderState,
        screen_binding: &ScreenBinding,
        object_binding: &ObjectBinding,
        profiler: &mut RenderProfiler,
    ) -> Self {
        let gpu_handle = render_state.gpu_handle.clone();

        let (color_texture, previous_color_texture) = Self::create_textures(render_state);

        let (texture_bind_group_layout, texture_bind_group) = Self::create_texture_binding(
            gpu_handle.clone(),
            &color_texture,
            &previous_color_texture,
        );

        let (lut_bind_group_layout, lut_bind_group) = Self::create_lut_binding(gpu_handle.clone());

        let pipeline_layout =
            gpu_handle
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pathtrace_pipeline_layout"),
                    bind_group_layouts: &[
                        &screen_binding.bind_group_layout,
                        &object_binding.bind_group_layout,
                        &texture_bind_group_layout,
                        &lut_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let (shader, pipeline) =
            Self::create_shader_and_pipeline(gpu_handle.clone(), &pipeline_layout);

        let time_query_index = profiler.push(&gpu_handle, "pathtrace");

        Self {
            color_texture,
            previous_color_texture,
            texture_bind_group_layout,
            texture_bind_group,
            lut_bind_group,
            shader,
            pipeline_layout,
            pipeline,
            time_query_index,
            _gpu_handle: gpu_handle,
        }
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        profiler: &mut RenderProfiler,
        screen_binding: &ScreenBinding,
        object_binding: &ObjectBinding,
    ) {
        let (_, time_query) = &mut profiler.time_queries[self.time_query_index];

        time_query.write_start_timestamp(encoder);

        // Copy current texture to previous texture
        encoder.copy_texture_to_texture(
            self.color_texture.inner().as_image_copy(),
            self.previous_color_texture.inner().as_image_copy(),
            self.color_texture.inner().size(),
        );

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pathtrace_compute_pass"),
            timestamp_writes: None,
        });

        compute_pass.set_bind_group(0, &screen_binding.bind_group, &[]);
        compute_pass.set_bind_group(1, &object_binding.bind_group, &[]);
        compute_pass.set_bind_group(2, &self.texture_bind_group, &[]);
        compute_pass.set_bind_group(3, &self.lut_bind_group, &[]);

        compute_pass.set_pipeline(&self.pipeline);

        let workgroup_sizes = UVec3::new(8, 8, 1);
        let dimensions = UVec3::new(
            self.color_texture.inner().width(),
            self.color_texture.inner().height(),
            1,
        );

        let mut workgroups = dimensions / workgroup_sizes;

        // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
        workgroups += (dimensions % workgroups) & UVec3::ONE;

        compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        drop(compute_pass);

        time_query.write_end_timestamp(encoder);
    }

    fn create_lut_binding(gpu_handle: GpuHandle) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let parent_path = std::env::current_dir().unwrap();

        let wavelength_to_xyz_texture = wgputil::texture::load_raw(
            &gpu_handle.device,
            &gpu_handle.queue,
            parent_path.join("assets/textures/wavelength_to_xyz.bin"),
            &wgpu::TextureDescriptor {
                label: Some("wavelength_to_xyz_texture"),
                size: wgpu::Extent3d {
                    width: 471,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
        )
        .unwrap();

        let rgb_to_spectral_intensity_texture = wgputil::texture::load_raw(
            &gpu_handle.device,
            &gpu_handle.queue,
            parent_path.join("assets/textures/rgb_to_spectral_intensity.bin"),
            &wgpu::TextureDescriptor {
                label: Some("rgb_to_spectral_intensity_texture"),
                size: wgpu::Extent3d {
                    width: 81,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
        )
        .unwrap();

        wgputil::binding::create_sequential_linked(
            &gpu_handle.device,
            "pathtrace_lut_binding",
            &[
                wgputil::binding::bind_texture_storage(
                    &wavelength_to_xyz_texture.create_view(&Default::default()),
                    wavelength_to_xyz_texture.format(),
                    wgpu::TextureViewDimension::D1,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                wgputil::binding::bind_texture_storage(
                    &rgb_to_spectral_intensity_texture.create_view(&Default::default()),
                    rgb_to_spectral_intensity_texture.format(),
                    wgpu::TextureViewDimension::D1,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
            ],
        )
    }

    fn create_textures(render_state: &RenderState) -> (Texture, Texture) {
        let texture_format = wgpu::TextureFormat::Rgba32Float;

        let color_texture = Texture::new(
            render_state.gpu_handle.clone(),
            "pathtrace_color_texture".into(),
            (
                render_state.get_effective_width() as usize,
                render_state.get_effective_height() as usize,
                1,
            ),
            1,
            texture_format,
            wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            TextureType::Texture2d,
        );

        let previous_color_texture = Texture::new(
            render_state.gpu_handle.clone(),
            "pathtrace_previous_color_texture".into(),
            (
                (render_state.effective_viewport_end.0 - render_state.effective_viewport_start.0)
                    as usize,
                (render_state.effective_viewport_end.1 - render_state.effective_viewport_start.1)
                    as usize,
                1,
            ),
            1,
            texture_format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture2d,
        );

        (color_texture, previous_color_texture)
    }

    fn create_texture_binding(
        gpu_handle: GpuHandle,
        color_texture: &Texture,
        previous_color_texture: &Texture,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        wgputil::binding::create_sequential_linked(
            &gpu_handle.device,
            "pathtrace_texture_binding",
            &[
                wgputil::binding::bind_texture_storage(
                    &color_texture.inner().create_view(&Default::default()),
                    color_texture.inner().format(),
                    wgpu::TextureViewDimension::D2,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                wgputil::binding::bind_texture_view(
                    &previous_color_texture
                        .inner()
                        .create_view(&Default::default()),
                    wgputil::texture::sample_type(
                        &gpu_handle.device,
                        previous_color_texture.inner(),
                    )
                    .unwrap(),
                    wgpu::TextureViewDimension::D2,
                ),
            ],
        )
    }

    fn create_shader_and_pipeline(
        gpu_handle: GpuHandle,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> (Shader, wgpu::ComputePipeline) {
        let shader = Shader::new(
            gpu_handle.clone(),
            ShaderSource::load_wgsl("assets/shaders/pathtrace.wgsl"),
        );

        let pipeline =
            gpu_handle
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("pathtrace_compute_pipeline"),
                    layout: Some(pipeline_layout),
                    module: shader.module(),
                    entry_point: Some("compute"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        (shader, pipeline)
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        screen_binding: Res<ScreenBinding>,
        object_binding: Res<ObjectBinding>,
        mut profiler: ResMut<RenderProfiler>,
    ) {
        let path_tracer = PathtracePass::new(
            &render_state,
            &screen_binding,
            &object_binding,
            &mut profiler,
        );

        commands.insert_resource(path_tracer);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut path_tracer: ResMut<PathtracePass>,
        render_state: Res<RenderState>,
        screen_binding: Res<ScreenBinding>,
        object_binding: Res<ObjectBinding>,
        mut profiler: ResMut<RenderProfiler>,

        mut frame: ResMut<FrameData>,

        mut resize_events: EventReader<WindowResizeEvent>,
        mut shader_recompile_events: EventReader<ShaderRecompileEvent>,
    ) {
        if resize_events.read().count() > 0 {
            let (color_texture, previous_color_texture) = Self::create_textures(&render_state);

            let texture_bind_group = wgputil::binding::create_sequential_with_layout(
                &render_state.gpu_handle.device,
                "pathtrace_texture_binding",
                &path_tracer.texture_bind_group_layout,
                &[
                    wgpu::BindingResource::TextureView(
                        &color_texture.inner().create_view(&Default::default()),
                    ),
                    wgpu::BindingResource::TextureView(
                        &previous_color_texture
                            .inner()
                            .create_view(&Default::default()),
                    ),
                ],
            );

            path_tracer.color_texture = color_texture;
            path_tracer.previous_color_texture = previous_color_texture;
            path_tracer.texture_bind_group = texture_bind_group;
        }

        if shader_recompile_events.read().count() > 0 {
            let (shader, pipeline) = Self::create_shader_and_pipeline(
                render_state.gpu_handle.clone(),
                &path_tracer.pipeline_layout,
            );

            path_tracer.shader = shader;
            path_tracer.pipeline = pipeline;
        }

        path_tracer.draw(
            &mut frame.encoder,
            &mut profiler,
            &screen_binding,
            &object_binding,
        );
    }
}
