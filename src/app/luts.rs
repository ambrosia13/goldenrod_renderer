use bevy_ecs::{
    resource::Resource,
    system::{Commands, Res},
};
use wgputil::GpuHandle;

use crate::{app::renderer::SurfaceState, util};

#[derive(Resource)]
pub struct SpectrumBinding {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl SpectrumBinding {
    fn new(gpu: &GpuHandle) -> Self {
        let wavelength_to_xyz_texture = wgputil::texture::load_raw(
            &gpu.device,
            &gpu.queue,
            util::asset_path("textures/spectrum/wavelength_to_xyz.bin"),
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
            &gpu.device,
            &gpu.queue,
            util::asset_path("textures/spectrum/rgb_to_spectral_intensity.bin"),
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

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &gpu.device,
            "spectrum_binding",
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
        );

        Self {
            bind_group_layout,
            bind_group,
        }
    }

    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        let spectrum_binding = SpectrumBinding::new(&surface_state.gpu);
        commands.insert_resource(spectrum_binding);
    }
}

#[derive(Resource)]
pub struct CameraResponseBinding {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraResponseBinding {
    fn new(gpu: &GpuHandle) -> Self {
        fn desc_supplier(name: &str) -> wgpu::TextureDescriptor<'_> {
            wgpu::TextureDescriptor {
                label: Some(name),
                size: wgpu::Extent3d {
                    width: 1024,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        }

        let camera_response_red = wgputil::texture::load_raw(
            &gpu.device,
            &gpu.queue,
            util::asset_path("textures/camera_response/Kodachrome-64CDRed.bin"),
            &desc_supplier("camera_response_red"),
        )
        .expect("Failed to read camera response texture");

        let camera_response_green = wgputil::texture::load_raw(
            &gpu.device,
            &gpu.queue,
            util::asset_path("textures/camera_response/Kodachrome-64CDGreen.bin"),
            &desc_supplier("camera_response_green"),
        )
        .expect("Failed to read camera response texture");

        let camera_response_blue = wgputil::texture::load_raw(
            &gpu.device,
            &gpu.queue,
            util::asset_path("textures/camera_response/Kodachrome-64CDBlue.bin"),
            &desc_supplier("camera_response_blue"),
        )
        .expect("Failed to read camera response texture");

        let camera_response_red_view =
            camera_response_red.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_response_green_view =
            camera_response_green.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_response_blue_view =
            camera_response_blue.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_response_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("camera_response_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &gpu.device,
            "camera_response_binding",
            &[
                wgputil::binding::bind_texture(
                    &camera_response_red_view,
                    wgputil::texture::sample_type(&gpu.device, &camera_response_red).unwrap(),
                    wgpu::TextureViewDimension::D1,
                ),
                wgputil::binding::bind_texture(
                    &camera_response_green_view,
                    wgputil::texture::sample_type(&gpu.device, &camera_response_green).unwrap(),
                    wgpu::TextureViewDimension::D1,
                ),
                wgputil::binding::bind_texture(
                    &camera_response_blue_view,
                    wgputil::texture::sample_type(&gpu.device, &camera_response_blue).unwrap(),
                    wgpu::TextureViewDimension::D1,
                ),
                wgputil::binding::BindingEntry {
                    binding_type: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                    resource: wgpu::BindingResource::Sampler(&camera_response_sampler),
                },
            ],
        );

        Self {
            bind_group_layout,
            bind_group,
        }
    }

    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        let camera_response_binding = CameraResponseBinding::new(&surface_state.gpu);
        commands.insert_resource(camera_response_binding);
    }
}
