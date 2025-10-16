use crate::app::bindless::{BindlessHeap, DescriptorHandle};
use crate::util;
use gpu_layout::AsGpuBytes;
use wgpu::util::DeviceExt;

#[derive(AsGpuBytes)]
pub struct LookupTable {
    handle: DescriptorHandle,
    min: f32,
    max: f32,
    default_value: f32,
}

impl LookupTable {
    pub fn load_from(
        device: wgpu::Device,
        bindless_heap: &mut BindlessHeap,
        asset_path: &str,
        min: f32,
        max: f32,
        default_value: f32,
    ) -> LookupTable {
        let path = util::asset_path(asset_path);
        let bytes = std::fs::read(path).unwrap();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(asset_path),
            contents: &bytes,
            usage: wgpu::BufferUsages::STORAGE,
        });

        let handle = bindless_heap.insert_buffer(&buffer);

        Self {
            handle,
            min,
            max,
            default_value,
        }
    }
}
