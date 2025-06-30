use bevy_ecs::resource::Resource;

use crate::app::{
    camera::binding::ScreenBinding,
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

    managed_pipeline: ManagedComputePipeline<'static>,

    time_query_index: usize,
}

// impl MaterialPass {
//     fn new(
//         surface_state: &SurfaceState,
//         renderer_viewport: &RendererViewport,
//         screen_binding: &ScreenBinding,
//         object_binding: &ObjectBinding,
//         profiler: &mut RenderProfiler,
//     ) -> Self {
//     }
// }
