use std::{collections::HashMap, sync::Arc};

use bevy_ecs::{
    resource::Resource,
    system::{Commands, Local, Res, ResMut},
};
use glam::UVec3;
use wgputil::shader::ShaderSource;

use crate::{
    app::{
        camera::binding::ScreenBinding,
        lookup::SpectrumBinding,
        object::binding::ObjectBinding,
        renderer::{profiler::RenderProfiler, FrameRecord, SurfaceState},
    },
    util,
};

#[derive(Resource)]
pub struct MaterialTextures {
    pub current_texture: wgpu::Texture,
    pub previous_texture: wgpu::Texture,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl MaterialTextures {
    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        let gpu = &surface_state.gpu;

        let (current_desc, previous_desc) = Self::desc(&surface_state);

        let current_texture = gpu.device.create_texture(&current_desc);
        let previous_texture = gpu.device.create_texture(&previous_desc);

        let current_texture_view = current_texture.create_view(&Default::default());
        let previous_texture_view = previous_texture.create_view(&Default::default());

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &gpu.device,
            "material_texture_binding",
            &[
                wgputil::binding::bind_storage_texture(
                    &current_texture_view,
                    current_texture.format(),
                    wgpu::TextureViewDimension::D2,
                    wgpu::StorageTextureAccess::WriteOnly,
                ),
                wgputil::binding::bind_storage_texture(
                    &previous_texture_view,
                    previous_texture.format(),
                    wgpu::TextureViewDimension::D2,
                    wgpu::StorageTextureAccess::ReadOnly,
                ),
            ],
        );

        commands.insert_resource(Self {
            current_texture,
            previous_texture,
            bind_group_layout,
            bind_group,
        });
    }

    pub fn on_resize(
        mut material_textures: ResMut<MaterialTextures>,
        surface_state: Res<SurfaceState>,
    ) {
        let gpu = &surface_state.gpu;

        let (current_desc, previous_desc) = Self::desc(&surface_state);

        let current_texture = gpu.device.create_texture(&current_desc);
        let previous_texture = gpu.device.create_texture(&previous_desc);

        let current_texture_view = current_texture.create_view(&Default::default());
        let previous_texture_view = previous_texture.create_view(&Default::default());

        let bind_group = wgputil::binding::create_sequential_with_layout(
            &surface_state.gpu.device,
            "material_texture_binding",
            &material_textures.bind_group_layout,
            &[
                wgpu::BindingResource::TextureView(&current_texture_view),
                wgpu::BindingResource::TextureView(&previous_texture_view),
            ],
        );

        material_textures.current_texture = current_texture;
        material_textures.previous_texture = previous_texture;
        material_textures.bind_group = bind_group;
    }

    fn desc(surface_state: &SurfaceState) -> (wgpu::TextureDescriptor, wgpu::TextureDescriptor) {
        let current_desc = wgpu::TextureDescriptor {
            label: Some("material_pass_current_texture"),
            size: wgpu::Extent3d {
                width: surface_state.config.width,
                height: surface_state.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let previous_desc = wgpu::TextureDescriptor {
            label: Some("material_pass_previous_texture"),
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ..current_desc
        };

        (current_desc, previous_desc)
    }
}

#[derive(Resource)]
pub struct MaterialPipelines {
    pipelines: HashMap<Arc<str>, wgpu::ComputePipeline>,
    active_pipeline: Arc<str>,
}

impl MaterialPipelines {
    pub fn init(
        mut commands: Commands,
        surface_state: Res<SurfaceState>,
        screen_binding: Res<ScreenBinding>,
        object_binding: Res<ObjectBinding>,
        spectrum_binding: Res<SpectrumBinding>,
        material_textures: Res<MaterialTextures>,
    ) {
        let gpu = &surface_state.gpu;

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("material_pipeline_layout"),
                bind_group_layouts: &[
                    &screen_binding.bind_group_layout,
                    &object_binding.bind_group_layout,
                    &spectrum_binding.bind_group_layout,
                    &material_textures.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let mut pipelines = HashMap::new();

        let pipeline_paths = ["pathtrace.slang"];
        let active_pipeline = Arc::from(pipeline_paths[0]);

        for pipeline_path in pipeline_paths {
            let path = util::shader_path(pipeline_path);

            let source = ShaderSource::load_spirv(&path);
            let module = wgputil::shader::create(&gpu.device, &source).unwrap();

            let pipeline = gpu
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(pipeline_path),
                    layout: Some(&layout),
                    module: &module,
                    entry_point: Some("compute"),
                    compilation_options: Default::default(),
                    cache: None,
                });

            pipelines.insert(Arc::from(pipeline_path), pipeline);
        }

        commands.insert_resource(Self {
            pipelines,
            active_pipeline,
        });
    }

    pub fn get_active_pipeline(&self) -> &wgpu::ComputePipeline {
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
    object_binding: Res<ObjectBinding>,
    spectrum_binding: Res<SpectrumBinding>,
    material_textures: Res<MaterialTextures>,

    material_pipelines: Res<MaterialPipelines>,
) {
    if time_query_index.is_none() {
        *time_query_index = Some(profiler.push(&surface_state.gpu, "material_pass"));
    }

    let (_, time_query) = &mut profiler.time_queries[time_query_index.unwrap()];

    // Copy current texture to previous texture
    frame.encoder.copy_texture_to_texture(
        material_textures.current_texture.as_image_copy(),
        material_textures.previous_texture.as_image_copy(),
        material_textures.current_texture.size(),
    );

    let mut compute_pass = frame
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("material_pass"),
            timestamp_writes: Some(time_query.compute_timestamp_writes()),
        });

    compute_pass.set_bind_group(0, &screen_binding.bind_group, &[]);
    compute_pass.set_bind_group(1, &object_binding.bind_group, &[]);
    compute_pass.set_bind_group(2, &spectrum_binding.bind_group, &[]);
    compute_pass.set_bind_group(3, &material_textures.bind_group, &[]);

    compute_pass.set_pipeline(material_pipelines.get_active_pipeline());

    let workgroup_sizes = UVec3::new(8, 8, 1);
    let dimensions = UVec3::new(
        material_textures.current_texture.width(),
        material_textures.current_texture.height(),
        1,
    );

    let mut workgroups = dimensions / workgroup_sizes;

    // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
    workgroups += (dimensions % workgroups) & UVec3::ONE;

    compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
}
