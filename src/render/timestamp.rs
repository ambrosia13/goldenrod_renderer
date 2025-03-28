use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use super::GpuHandle;

pub struct TimeQuery {
    started: bool,

    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_buffer: Arc<wgpu::Buffer>,

    gpu_handle: GpuHandle,
}

impl TimeQuery {
    pub fn new(gpu_handle: impl Into<GpuHandle>) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        let query_set = gpu_handle
            .device
            .create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                ty: wgpu::QueryType::Timestamp,
                count: 2, // one for before timestamp, one for after
            });

        let resolve_buffer = gpu_handle.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 2 * 8, // 2 u64s, 8 bytes each
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = gpu_handle.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 2 * 8, // 2 u64s, 8 bytes each
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let readback_buffer = Arc::new(readback_buffer);

        Self {
            started: false,
            query_set,
            resolve_buffer,
            readback_buffer,
            gpu_handle,
        }
    }

    pub fn write_start_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.started {
            panic!("Attempted to write a start timestamp more than once");
        }

        self.started = true;
        encoder.write_timestamp(&self.query_set, 0);
    }

    pub fn write_end_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.started {
            panic!("Attempted to write an end timestamp without first starting");
        }

        self.started = false;
        encoder.write_timestamp(&self.query_set, 1);

        // after the timestamp is written, resolve the query and prepare for readback
        //self.resolve(encoder);
    }

    fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.query_set, 0..2, &self.resolve_buffer, 0);

        // Copy the data to a mapped buffer so it can be read on the cpu
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.readback_buffer,
            0,
            self.resolve_buffer.size(),
        );
    }

    pub fn read(&self) -> Duration {
        let mut encoder = self
            .gpu_handle
            .device
            .create_command_encoder(&Default::default());

        // resolve with temporary command encoder instead of the frame encoder
        self.resolve(&mut encoder);

        self.gpu_handle
            .queue
            .submit(std::iter::once(encoder.finish()));

        let (tx, rx) = mpsc::channel();

        let buffer = self.readback_buffer.clone();

        self.readback_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    let view = buffer.slice(..).get_mapped_range();
                    let timestamps: &[u64] = bytemuck::cast_slice(&view);

                    let time_start = timestamps[0];
                    let time_end = timestamps[1];

                    tx.send((time_start, time_end)).unwrap();
                }

                buffer.unmap();
            });

        self.gpu_handle.device.poll(wgpu::MaintainBase::Wait);

        let (start, end) = rx.recv().unwrap();

        let timestamp_period = self.gpu_handle.queue.get_timestamp_period() as f64;
        let nanoseconds = (end - start) as f64 * timestamp_period;

        Duration::from_nanos(nanoseconds as u64)
    }
}
