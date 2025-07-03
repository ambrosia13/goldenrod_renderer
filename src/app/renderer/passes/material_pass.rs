use bevy_ecs::resource::Resource;

use crate::app::{
    camera::binding::ScreenBinding,
    lookup::SpectrumBinding,
    object::binding::ObjectBinding,
    renderer::{
        pipelines::ManagedComputePipeline, profiler::RenderProfiler, RendererViewport, SurfaceState,
    },
};

#[derive(Resource)]
pub struct MaterialPass {
    pub color_texture: wgpu::Texture,
    previous_color_texture: wgpu::Texture,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    lut_bind_group: wgpu::BindGroup,

    pipeline_layout: wgpu::PipelineLayout,
    managed_pipeline: ManagedComputePipeline<'static>,

    time_query_index: usize,
}

impl MaterialPass {
    fn new(
        surface_state: &SurfaceState,
        renderer_viewport: &RendererViewport,

        screen_binding: &ScreenBinding,
        object_binding: &ObjectBinding,
        spectrum_binding: &SpectrumBinding,

        profiler: &mut RenderProfiler,
    ) -> Self {
        let gpu = &surface_state.gpu;

        todo!()
        //let texture_desc = Self::get_texture_descriptor(name, surface_state, copy_dst)
    }

    // fn get_texture_descriptor<'a>(
    //     name: &'a str,
    //     surface_state: &SurfaceState,
    //     copy_dst: bool,
    // ) -> wgpu::TextureDescriptor<'a> {
    //     wgpu::TextureDescriptor {
    //         label: Some(name),
    //         size: wgpu::Extent3d {
    //             width: surface_state.config.width,
    //             height: surface_state.config.height,
    //             depth_or_array_layers: 1,
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: wgpu::TextureFormat::Rgba32Float,
    //         usage: wgpu::TextureUsages::STORAGE_BINDING
    //             | if copy_dst {
    //                 wgpu::TextureUsages::COPY_DST
    //             } else {
    //                 wgpu::TextureUsages::COPY_SRC
    //             },
    //         view_formats: &[],
    //     }
    // }
}
