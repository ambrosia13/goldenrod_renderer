use crate::app::bindless::{BINDLESS_BUFFER_COUNT, BINDLESS_SAMPLER_COUNT, BINDLESS_TEXTURE_COUNT};
use env_logger::Env;

mod app;
mod ecs;
mod egui;
mod util;

pub const WGPU_FEATURES: wgpu::Features =
    // general extensions
    wgpu::Features::FLOAT32_FILTERABLE
        .union(wgpu::Features::RG11B10UFLOAT_RENDERABLE)
        .union(wgpu::Features::PUSH_CONSTANTS)
        .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER)
        .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO)
        .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
        // bindless resources
        .union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY)
        .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
        .union(wgpu::Features::BUFFER_BINDING_ARRAY)
        .union(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY) // for storage buffer binding
        .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
        // for performance metrics
        .union(wgpu::Features::TIMESTAMP_QUERY)
        .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
        // so we can have binding visibility in all shader stages
        .union(wgpu::Features::VERTEX_WRITABLE_STORAGE)
        // for slang shader compat
        .union(wgpu::Features::SPIRV_SHADER_PASSTHROUGH);

// Can't be constant value because Default::default() isn't const
pub fn wgpu_limits() -> wgpu::Limits {
    wgpu::Limits {
        max_push_constant_size: 128,
        max_binding_array_elements_per_shader_stage: BINDLESS_TEXTURE_COUNT
            + BINDLESS_BUFFER_COUNT
            + BINDLESS_SAMPLER_COUNT,
        max_binding_array_sampler_elements_per_shader_stage: BINDLESS_SAMPLER_COUNT,
        ..Default::default()
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("goldenrod", log::LevelFilter::Info)
        .init();

    app::run();
}
