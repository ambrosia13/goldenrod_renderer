use bevy_ecs::{
    change_detection::DetectChanges,
    resource::Resource,
    system::{Commands, Res, ResMut},
};
use gpu_bytes::AsStd430;
use wgpu::util::DeviceExt;

use crate::app::{
    object::{Aabb, Material, Sphere, Triangle},
    renderer::SurfaceState,
};

use super::Objects;

#[derive(Resource)]
pub struct ObjectBinding {
    pub materials_buffer: wgpu::Buffer,
    pub spheres_buffer: wgpu::Buffer,
    pub aabbs_buffer: wgpu::Buffer,
    pub triangles_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ObjectBinding {
    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        // create empty buffers at first
        let create_buffer = |name: &str, data: &[u8]| {
            surface_state
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(name),
                    contents: data,
                    usage: wgpu::BufferUsages::STORAGE,
                })
        };

        let materials = vec![Material::null()];
        let spheres = vec![Sphere::null()];
        let aabbs = vec![Aabb::null()];
        let triangles = vec![Triangle::null()];

        let materials_buffer = create_buffer("materials_buffer", materials.as_std430().as_slice());
        let spheres_buffer = create_buffer("spheres_buffer", spheres.as_std430().as_slice());
        let aabbs_buffer = create_buffer("aabbs_buffer", aabbs.as_std430().as_slice());
        let triangles_buffer = create_buffer("triangles_buffer", triangles.as_std430().as_slice());

        let (bind_group_layout, bind_group) = wgputil::binding::create_sequential_linked(
            &surface_state.gpu.device,
            "object_binding",
            &[
                wgputil::binding::bind_buffer_storage(&materials_buffer, true),
                wgputil::binding::bind_buffer_storage(&spheres_buffer, true),
                wgputil::binding::bind_buffer_storage(&aabbs_buffer, true),
                wgputil::binding::bind_buffer_storage(&triangles_buffer, true),
            ],
        );

        let object_binding = Self {
            materials_buffer,
            spheres_buffer,
            aabbs_buffer,
            triangles_buffer,
            bind_group_layout,
            bind_group,
        };

        commands.insert_resource(object_binding);
    }

    pub fn update(
        surface_state: Res<SurfaceState>,
        mut object_binding: ResMut<ObjectBinding>,
        objects: Res<Objects>,
    ) {
        // check if we need to update
        if !objects.is_changed() {
            return;
        }

        // Pad the original list values with null to avoid empty buffers
        let mut materials = vec![Material::null()];
        materials.extend_from_slice(&objects.materials);

        let mut spheres = vec![Sphere::null()];
        spheres.extend_from_slice(&objects.spheres);

        let mut aabbs = vec![Aabb::null()];
        aabbs.extend_from_slice(&objects.aabbs);

        let mut triangles = vec![Triangle::null()];
        triangles.extend_from_slice(&objects.triangles);

        let usage = wgpu::BufferUsages::STORAGE;

        let device = &surface_state.gpu.device;
        object_binding.materials_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("materials_buffer"),
                contents: materials.as_std430().as_slice(),
                usage,
            });

        object_binding.spheres_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spheres_buffer"),
                contents: spheres.as_std430().as_slice(),
                usage,
            });

        object_binding.aabbs_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("aabbs_buffer"),
                contents: aabbs.as_std430().as_slice(),
                usage,
            });

        object_binding.triangles_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("triangles_buffer"),
                contents: triangles.as_std430().as_slice(),
                usage,
            });

        object_binding.bind_group = wgputil::binding::create_sequential_with_layout(
            device,
            "object_binding",
            &object_binding.bind_group_layout,
            &[
                object_binding.materials_buffer.as_entire_binding(),
                object_binding.spheres_buffer.as_entire_binding(),
                object_binding.aabbs_buffer.as_entire_binding(),
                object_binding.triangles_buffer.as_entire_binding(),
            ],
        );
    }
}
