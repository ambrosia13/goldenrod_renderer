// use bytemuck::{Pod, Zeroable};
// use glam::Vec2;

// pub trait VertexData: bytemuck::Pod + bytemuck::NoUninit {
//     fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
// }

// pub trait InstanceData {
//     fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
// }

// #[repr(C)]
// #[derive(Pod, Zeroable, Clone, Copy)]
// pub struct FrameVertex {
//     position: Vec2,
//     uv: Vec2,
//     texcoord: Vec2,
// }

// impl FrameVertex {
//     const VERTICES: &'static [Self] = &[
//         Self {
//             position: Vec2::new(-1.0, -1.0),
//             uv: Vec2::new(0.0, 1.0),
//             texcoord: Vec2::new(0.0, 0.0),
//         },
//         Self {
//             position: Vec2::new(1.0, -1.0),
//             uv: Vec2::new(1.0, 1.0),
//             texcoord: Vec2::new(1.0, 0.0),
//         },
//         Self {
//             position: Vec2::new(1.0, 1.0),
//             uv: Vec2::new(1.0, 0.0),
//             texcoord: Vec2::new(1.0, 1.0),
//         },
//         Self {
//             position: Vec2::new(-1.0, 1.0),
//             uv: Vec2::new(0.0, 0.0),
//             texcoord: Vec2::new(0.0, 1.0),
//         },
//     ];

//     const INDICES: &'static [u32] = &[0, 1, 2, 0, 2, 3];
// }

// pub struct FrameQuad {
//     pub vertex_buffer:
// }
