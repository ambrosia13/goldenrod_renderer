use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Res, ResMut};
use glam::UVec3;
use wgputil::GpuHandle;

use crate::app::camera::binding::ScreenBinding;
use crate::app::object::binding::ObjectBinding;
use crate::app::renderer::{FrameRecord, RendererViewport, SurfaceState};
use crate::util;

use super::profiler::RenderProfiler;

#[derive(Resource)]
pub struct PathtracePass {
    pub color_texture: wgpu::Texture,
    pub previous_color_texture: wgpu::Texture,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,

    lut_bind_group: wgpu::BindGroup,

    shader_source: wgputil::shader::ShaderSource,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,

    time_query_index: usize,
}

impl PathtracePass {
    fn new(
        surface_state: &SurfaceState,
        renderer_viewport: &RendererViewport,
        screen_binding: &ScreenBinding,
        object_binding: &ObjectBinding,
        profiler: &mut RenderProfiler,
    ) -> Self {
        let gpu_handle = surface_state.gpu.clone();

        let (color_texture, previous_color_texture) =
            Self::create_textures(surface_state, renderer_viewport);

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

        let (shader_source, shader) = Self::create_shader(&gpu_handle.device);
        let pipeline = Self::create_pipeline(&gpu_handle.device, &shader, &pipeline_layout);

        let time_query_index = profiler.push(&gpu_handle, "pathtrace");

        Self {
            color_texture,
            previous_color_texture,
            texture_bind_group_layout,
            texture_bind_group,
            lut_bind_group,
            shader_source,
            shader,
            pipeline_layout,
            pipeline,
            time_query_index,
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

        // Copy current texture to previous texture
        encoder.copy_texture_to_texture(
            self.color_texture.as_image_copy(),
            self.previous_color_texture.as_image_copy(),
            self.color_texture.size(),
        );

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pathtrace_compute_pass"),
            timestamp_writes: Some(time_query.compute_timestamp_writes()),
        });

        compute_pass.set_bind_group(0, &screen_binding.bind_group, &[]);
        compute_pass.set_bind_group(1, &object_binding.bind_group, &[]);
        compute_pass.set_bind_group(2, &self.texture_bind_group, &[]);
        compute_pass.set_bind_group(3, &self.lut_bind_group, &[]);

        compute_pass.set_pipeline(&self.pipeline);

        let workgroup_sizes = UVec3::new(8, 8, 1);
        let dimensions = UVec3::new(self.color_texture.width(), self.color_texture.height(), 1);

        let mut workgroups = dimensions / workgroup_sizes;

        // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
        workgroups += (dimensions % workgroups) & UVec3::ONE;

        compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        drop(compute_pass);
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
                wgputil::binding::bind_storage_texture(
                    &wavelength_to_xyz_texture.create_view(&Default::default()),
                    wavelength_to_xyz_texture.format(),
                    wgpu::TextureViewDimension::D1,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                wgputil::binding::bind_storage_texture(
                    &rgb_to_spectral_intensity_texture.create_view(&Default::default()),
                    rgb_to_spectral_intensity_texture.format(),
                    wgpu::TextureViewDimension::D1,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
            ],
        )
    }

    fn create_textures(
        surface_state: &SurfaceState,
        renderer_viewport: &RendererViewport,
    ) -> (wgpu::Texture, wgpu::Texture) {
        let texture_format = wgpu::TextureFormat::Rgba32Float;

        let color_texture = surface_state
            .gpu
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("pathtrace_color_texture"),
                size: wgpu::Extent3d {
                    width: renderer_viewport.get_width(),
                    height: renderer_viewport.get_height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: texture_format,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let previous_color_texture =
            surface_state
                .gpu
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("pathtrace_previous_color_texture"),
                    size: color_texture.size(),
                    mip_level_count: color_texture.mip_level_count(),
                    sample_count: color_texture.sample_count(),
                    dimension: color_texture.dimension(),
                    format: color_texture.format(),
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

        (color_texture, previous_color_texture)
    }

    fn create_texture_binding(
        gpu_handle: GpuHandle,
        color_texture: &wgpu::Texture,
        previous_color_texture: &wgpu::Texture,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        wgputil::binding::create_sequential_linked(
            &gpu_handle.device,
            "pathtrace_texture_binding",
            &[
                wgputil::binding::bind_storage_texture(
                    &color_texture.create_view(&Default::default()),
                    color_texture.format(),
                    wgpu::TextureViewDimension::D2,
                    wgpu::StorageTextureAccess::ReadWrite,
                ),
                wgputil::binding::bind_texture(
                    &previous_color_texture.create_view(&Default::default()),
                    wgputil::texture::sample_type(&gpu_handle.device, previous_color_texture)
                        .unwrap(),
                    wgpu::TextureViewDimension::D2,
                ),
            ],
        )
    }

    fn create_shader(device: &wgpu::Device) -> (wgputil::shader::ShaderSource, wgpu::ShaderModule) {
        let mut shader_source =
            wgputil::shader::ShaderSource::load_spirv(util::shader_path("pathtrace.slang"));

        let (shader, _error) = wgputil::shader::create_or_fallback(device, &mut shader_source);

        (shader_source, shader)
    }

    fn create_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::ComputePipeline {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pathtrace_compute_pipeline"),
            layout: Some(pipeline_layout),
            module: shader,
            entry_point: Some("compute"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }

    pub fn init(
        mut commands: Commands,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
        screen_binding: Res<ScreenBinding>,
        object_binding: Res<ObjectBinding>,
        mut profiler: ResMut<RenderProfiler>,
    ) {
        let path_tracer = PathtracePass::new(
            &surface_state,
            &renderer_viewport,
            &screen_binding,
            &object_binding,
            &mut profiler,
        );

        commands.insert_resource(path_tracer);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        path_tracer: Res<PathtracePass>,
        screen_binding: Res<ScreenBinding>,
        object_binding: Res<ObjectBinding>,
        mut profiler: ResMut<RenderProfiler>,

        mut frame: ResMut<FrameRecord>,
    ) {
        path_tracer.draw(
            &mut frame.encoder,
            &mut profiler,
            &screen_binding,
            &object_binding,
        );
    }

    pub fn on_resize(
        mut path_tracer: ResMut<PathtracePass>,
        surface_state: Res<SurfaceState>,
        renderer_viewport: Res<RendererViewport>,
    ) {
        let (color_texture, previous_color_texture) =
            Self::create_textures(&surface_state, &renderer_viewport);

        let texture_bind_group = wgputil::binding::create_sequential_with_layout(
            &surface_state.gpu.device,
            "pathtrace_texture_binding",
            &path_tracer.texture_bind_group_layout,
            &[
                wgpu::BindingResource::TextureView(&color_texture.create_view(&Default::default())),
                wgpu::BindingResource::TextureView(
                    &previous_color_texture.create_view(&Default::default()),
                ),
            ],
        );

        path_tracer.color_texture = color_texture;
        path_tracer.previous_color_texture = previous_color_texture;
        path_tracer.texture_bind_group = texture_bind_group;
    }
}
