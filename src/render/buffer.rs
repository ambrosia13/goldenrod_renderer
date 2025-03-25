use std::{marker::PhantomData, num::NonZero};

use bevy_ecs::{component::Component, entity::Entity};
use gpu_bytes::{AsStd430, Std430Bytes};
use wgpu::util::DeviceExt;

use super::{binding::BindingEntry, GpuHandle};

#[derive(Component)]
pub struct Buffer {
    buffer: wgpu::Buffer,
    size: usize,

    gpu_handle: GpuHandle,
}

impl Buffer {
    pub fn with_data<'a>(
        gpu_handle: impl Into<GpuHandle>,
        name: &'a str,
        data: &'a [u8],
        usage: wgpu::BufferUsages,
    ) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let buffer = gpu_handle
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: data,
                usage,
            });

        Self {
            buffer,
            size: data.len(),
            gpu_handle,
        }
    }

    pub fn with_size(
        gpu_handle: impl Into<GpuHandle>,
        name: &str,
        size: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let buffer = gpu_handle.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(name),
            size: size as u64,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            size,
            gpu_handle,
        }
    }

    pub fn write(&self, data: &[u8], offset: usize) {
        if data.len() + offset > self.size {
            panic!("Attempted to write {} bytes of data at an offset of {} bytes which would overflow buffer size of {} bytes", data.len(), offset, self.size);
        }

        self.gpu_handle
            .queue
            .write_buffer_with(
                &self.buffer,
                offset as u64,
                NonZero::new(data.len() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(data);
    }

    pub fn bind_uniform(
        &self,
        offset: usize,
        size: Option<usize>,
        has_dynamic_offset: bool,
    ) -> BindingEntry {
        BindingEntry {
            binding_type: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset,
                min_binding_size: None,
            },
            count: None,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: offset as u64,
                size: size.map(|s| s as u64).and_then(NonZero::new),
            }),
        }
    }

    pub fn bind_storage(
        &self,
        read_only: bool,
        offset: usize,
        size: Option<usize>,
        has_dynamic_offset: bool,
    ) -> BindingEntry {
        BindingEntry {
            binding_type: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset,
                min_binding_size: None,
            },
            count: None,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: offset as u64,
                size: size.map(|s| s as u64).and_then(NonZero::new),
            }),
        }
    }
}

#[derive(Component)]
pub struct BufferVec<T: AsStd430 + Default> {
    inner: Buffer,
    name: String,
    _phantom: PhantomData<T>,
}

impl<T: AsStd430 + Default> BufferVec<T> {
    fn buffer_usages() -> wgpu::BufferUsages {
        // We can only have storage buffers be unsized in the shader,
        // so this must be a storage buffer
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
    }

    pub fn empty(gpu_handle: impl Into<GpuHandle>, name: &str) -> Self {
        let mut buf = Std430Bytes::new();

        // // Length (0)
        // buf.write(&0u32);

        // One garbage element since we can't have empty arrays in wgsl
        buf.write(&T::default());

        let inner = Buffer::with_data(gpu_handle, name, buf.as_slice(), Self::buffer_usages());

        Self {
            inner,
            name: name.to_owned(),
            _phantom: PhantomData,
        }
    }

    pub fn with_data(gpu_handle: impl Into<GpuHandle>, name: &str, data: &[T]) -> Self {
        if data.is_empty() {
            return Self::empty(gpu_handle, name);
        }

        let mut buf = Std430Bytes::new();

        // // Length
        // buf.write(&(data.len() as u32));

        // Data
        buf.write_array(data);

        let inner = Buffer::with_data(gpu_handle, name, buf.as_slice(), Self::buffer_usages());

        Self {
            inner,
            name: name.to_owned(),
            _phantom: PhantomData,
        }
    }

    /// Returns true if the buffer reallocated
    pub fn copy_from(&mut self, data: &[T]) -> bool {
        let mut buf = Std430Bytes::new();

        // // Length (0 if empty)
        // buf.write(&(data.len() as u32));

        // If the array isnt empty, write the data. otherwise write a single garbage value
        if !data.is_empty() {
            buf.write_array(data);
        } else {
            buf.write(&T::default());
        }

        // Needs to reallocate
        if buf.as_slice().len() > self.inner.size {
            *self = Self {
                inner: Buffer::with_data(
                    self.inner.gpu_handle.clone(),
                    &self.name,
                    buf.as_slice(),
                    Self::buffer_usages(),
                ),
                name: self.name.clone(),
                _phantom: PhantomData,
            };

            true
        } else {
            // Doesn't need to reallocate, we can write as normal
            self.inner.write(buf.as_slice(), 0);

            false
        }
    }

    // todo: finish later, for now just write to buffer every time
    // pub fn push(&mut self, data: &T) -> Self {
    //     let buf = data.as_std430();

    //     let element_size = buf.as_slice().len();
    //     // Subtract 4
    //     let existing_element_count = (self.inner.size - 4) - element_size;
    // }

    pub fn bind(&self, read_only: bool) -> BindingEntry {
        BindingEntry {
            binding_type: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
            resource: self.inner.buffer.as_entire_binding(),
        }
    }
}
