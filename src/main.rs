use env_logger::Env;

mod app;
mod ecs;
mod egui;
mod util;

pub const WGPU_FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE
    .union(wgpu::Features::RG11B10UFLOAT_RENDERABLE)
    .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
    .union(wgpu::Features::PUSH_CONSTANTS)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO)
    .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
    .union(wgpu::Features::TIMESTAMP_QUERY)
    .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
    .union(wgpu::Features::VERTEX_WRITABLE_STORAGE)
    .union(wgpu::Features::SPIRV_SHADER_PASSTHROUGH);

// Can't be constant value because Default::default() isn't const
pub fn wgpu_limits() -> wgpu::Limits {
    wgpu::Limits {
        max_push_constant_size: 128,
        ..Default::default()
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("goldenrod", log::LevelFilter::Info)
        .init();

    app::run();
}
