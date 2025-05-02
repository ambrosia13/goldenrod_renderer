use std::{ops::Range, path::Path, sync::Arc};

use bevy_ecs::component::Component;

use crate::util;

use super::{binding::BindingEntry, GpuHandle, WGPU_FEATURES};

#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    Texture1d,

    Texture2d,
    Texture2dArray,
    TextureCube,
    TextureCubeArray,

    Texture3d,
}

impl TextureType {
    pub fn dimension(self) -> wgpu::TextureDimension {
        match self {
            TextureType::Texture1d => wgpu::TextureDimension::D1,
            TextureType::Texture2d => wgpu::TextureDimension::D2,
            TextureType::Texture2dArray => wgpu::TextureDimension::D2,
            TextureType::TextureCube => wgpu::TextureDimension::D2,
            TextureType::TextureCubeArray => wgpu::TextureDimension::D2,
            TextureType::Texture3d => wgpu::TextureDimension::D3,
        }
    }

    pub fn view_dimension(self) -> wgpu::TextureViewDimension {
        match self {
            TextureType::Texture1d => wgpu::TextureViewDimension::D1,
            TextureType::Texture2d => wgpu::TextureViewDimension::D2,
            TextureType::Texture2dArray => wgpu::TextureViewDimension::D2Array,
            TextureType::TextureCube => wgpu::TextureViewDimension::Cube,
            TextureType::TextureCubeArray => wgpu::TextureViewDimension::CubeArray,
            TextureType::Texture3d => wgpu::TextureViewDimension::D3,
        }
    }
}

/// wgpu::TextureDescriptor but without lifetime
pub struct TextureDescriptor {
    pub label: Arc<str>,
    pub size: wgpu::Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: wgpu::TextureDimension,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub view_formats: Vec<wgpu::TextureFormat>,
}

impl TextureDescriptor {
    pub fn to_wgpu(&self) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label: Some(&self.label),
            size: self.size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: &self.view_formats,
        }
    }
}

#[derive(Component)]
pub struct Texture {
    inner: wgpu::Texture,
    desc: TextureDescriptor,
    ty: TextureType,

    name: Arc<str>,
    gpu_handle: GpuHandle,
}

impl Texture {
    pub fn load_raw<P: AsRef<Path>>(
        gpu_handle: impl Into<GpuHandle>,
        path: P,
        size: (usize, usize, usize),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        ty: TextureType,
    ) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let name: Arc<str> = Arc::from(util::path_name_to_string(&path).into_boxed_str());

        let texture = Self::new(gpu_handle.clone(), name, size, 1, format, usage, ty);

        let parent_path = std::env::current_dir().unwrap();
        let path = parent_path.join(path);

        let bytes = std::fs::read(path).expect("Failed to read texture from path");

        let bytes_per_pixel = format.block_copy_size(None).unwrap();
        let bytes_per_row = bytes_per_pixel * (size.0 as u32);

        let rows_per_image = match ty.dimension() {
            wgpu::TextureDimension::D3 => Some(size.1 as u32),
            _ => None,
        };

        gpu_handle.queue.write_texture(
            texture.inner.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image,
            },
            wgpu::Extent3d {
                width: size.0 as u32,
                height: size.1 as u32,
                depth_or_array_layers: size.2 as u32,
            },
        );

        texture
    }

    pub fn new(
        gpu_handle: impl Into<GpuHandle>,
        name: Arc<str>,
        size: (usize, usize, usize),
        mip_count: usize,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        ty: TextureType,
    ) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let desc = TextureDescriptor {
            label: name,
            size: wgpu::Extent3d {
                width: size.0 as u32,
                height: size.1 as u32,
                depth_or_array_layers: size.2 as u32,
            },
            mip_level_count: mip_count as u32,
            sample_count: 1,
            dimension: ty.dimension(),
            format,
            usage,
            view_formats: vec![],
        };

        let inner = gpu_handle.device.create_texture(&desc.to_wgpu());

        Self {
            inner,
            name: desc.label.clone(),
            desc,
            ty,
            gpu_handle,
        }
    }

    pub fn inner(&self) -> &wgpu::Texture {
        &self.inner
    }

    pub fn write(
        &self,
        data: &[u8],
        size: (usize, usize, usize),
        offset: (usize, usize, usize),
        target_mip: u32,
    ) {
        let bytes_per_pixel = self.desc.format.target_pixel_byte_cost().expect("Tried to write into a texture with a format that should probably not be manually written to");

        self.gpu_handle.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.inner,
                mip_level: target_mip,
                origin: wgpu::Origin3d {
                    x: offset.0 as u32,
                    y: offset.1 as u32,
                    z: offset.2 as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.0 as u32 * bytes_per_pixel),
                rows_per_image: Some(size.1 as u32),
            },
            wgpu::Extent3d {
                width: size.0 as u32,
                height: size.1 as u32,
                depth_or_array_layers: size.2 as u32,
            },
        );
    }

    pub fn write_whole(&self, data: &[u8], target_mip: u32) {
        let mip_dimensions = (
            (self.desc.size.width >> target_mip).max(1) as usize,
            (self.desc.size.height >> target_mip).max(1) as usize,
            (self.desc.size.depth_or_array_layers >> target_mip).max(1) as usize,
        );

        self.write(data, mip_dimensions, (0, 0, 0), target_mip);
    }

    pub fn view(&self, mip_range: Range<u32>, layer_range: Range<u32>) -> wgpu::TextureView {
        self.inner.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} View", self.name)),
            format: Some(self.desc.format),
            dimension: Some(self.ty.view_dimension()),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: mip_range.start,
            mip_level_count: Some(mip_range.end - mip_range.start),
            base_array_layer: layer_range.start,
            array_layer_count: Some(layer_range.end - layer_range.start),
            usage: None,
        })
    }

    pub fn recreate(&mut self) {
        self.inner = self.gpu_handle.device.create_texture(&self.desc.to_wgpu());
    }

    pub fn bind_view<'a>(&self, view: &'a wgpu::TextureView) -> BindingEntry<'a> {
        BindingEntry {
            binding_type: wgpu::BindingType::Texture {
                sample_type: self
                    .inner
                    .format()
                    .sample_type(None, Some(WGPU_FEATURES))
                    .unwrap(),
                view_dimension: self.ty.view_dimension(),
                multisampled: false,
            },
            count: None,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }

    pub fn bind_storage<'a>(
        &self,
        view: &'a wgpu::TextureView,
        access: wgpu::StorageTextureAccess,
    ) -> BindingEntry<'a> {
        BindingEntry {
            binding_type: wgpu::BindingType::StorageTexture {
                access,
                format: self.desc.format,
                view_dimension: self.ty.view_dimension(),
            },
            count: None,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }
}

pub struct Sampler {
    inner: wgpu::Sampler,
}

// /// Indicates that the size of the texture is tied to the screen
// #[derive(Component)]
// pub struct ScreenSizeTexture;

// pub fn update_screen_size_textures(
//     mut texture_query: Query<&mut Texture, With<ScreenSizeTexture>>,
//     mut resize_events: EventReader<WindowResizeEvent>,
//     menu: Res<Menu>,
// ) {
//     // If there's no resize events, we don't need to update the textures
//     if resize_events.read().count() == 0 {
//         return;
//     }

//     let new_width = menu.central_viewport_end.0 - menu.central_viewport_start.0;
//     let new_height = menu.central_viewport_end.1 - menu.central_viewport_start.1;

//     for mut texture in texture_query.iter_mut() {
//         texture.desc.size.width = new_width;
//         texture.desc.size.height = new_height;

//         texture.recreate();
//     }
// }
