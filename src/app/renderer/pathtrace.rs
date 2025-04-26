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
        binding::Binding,
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

    texture_binding: Binding,
    lut_binding: Binding,

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

        let texture_binding = Self::create_texture_binding(
            gpu_handle.clone(),
            &color_texture,
            &previous_color_texture,
        );

        let lut_binding = Self::create_lut_binding(gpu_handle.clone());

        let pipeline_layout =
            gpu_handle
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pathtrace_pipeline_layout"),
                    bind_group_layouts: &[
                        &screen_binding.bind_group_layout,
                        &object_binding.bind_group_layout,
                        texture_binding.bind_group_layout(),
                        lut_binding.bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let (shader, pipeline) =
            Self::create_shader_and_pipeline(gpu_handle.clone(), &pipeline_layout);

        let time_query_index = profiler.push(&gpu_handle, "pathtrace");

        Self {
            color_texture,
            previous_color_texture,
            texture_binding,
            lut_binding,
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
        compute_pass.set_bind_group(2, self.texture_binding.bind_group(), &[]);
        compute_pass.set_bind_group(3, self.lut_binding.bind_group(), &[]);

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

    fn create_lut_binding(gpu_handle: GpuHandle) -> Binding {
        let wavelength_to_xyz_texture = Texture::load_raw(
            gpu_handle.clone(),
            "assets/textures/wavelength_to_xyz.bin",
            (471, 1, 1),
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture1d,
        );

        let rgb_to_spectral_intensity_texture = Texture::load_raw(
            gpu_handle.clone(),
            "assets/textures/rgb_to_spectral_intensity.bin",
            (81, 1, 1),
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Texture1d,
        );

        Binding::new(
            gpu_handle,
            &[
                wavelength_to_xyz_texture.bind_storage(
                    &wavelength_to_xyz_texture.view(0..1, 0..1),
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                rgb_to_spectral_intensity_texture.bind_storage(
                    &rgb_to_spectral_intensity_texture.view(0..1, 0..1),
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
    ) -> Binding {
        Binding::new(
            gpu_handle,
            &[
                color_texture.bind_storage(
                    &color_texture.view(0..1, 0..1),
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                previous_color_texture.bind_view(&previous_color_texture.view(0..1, 0..1)),
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
            let texture_binding = Self::create_texture_binding(
                render_state.gpu_handle.clone(),
                &color_texture,
                &previous_color_texture,
            );

            path_tracer.color_texture = color_texture;
            path_tracer.previous_color_texture = previous_color_texture;
            path_tracer.texture_binding = texture_binding;
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
