use bevy_ecs::event::Event;
use bevy_ecs::resource::Resource;
use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Commands, Res, ResMut},
};
use glam::Vec3;
use gpu_bytes::AsStd430;
use gpu_bytes_derive::AsStd430;

use crate::render::{buffer::BufferVec, RenderState};

#[derive(Event)]
pub struct MaterialPushEvent(pub Material);

#[derive(Event)]
pub struct MaterialPopEvent;

#[derive(Event)]
pub struct SpherePushEvent(pub Sphere);

#[derive(Event)]
pub struct SpherePopEvent;

#[derive(Event)]
pub struct AabbPushEvent(pub Aabb);

#[derive(Event)]
pub struct AabbPopEvent;

#[derive(Event)]
pub struct TrianglePushEvent(pub Triangle);

#[derive(Event)]
pub struct TrianglePopEvent;

#[derive(Resource)]
pub struct Objects {
    pub materials: Materials,
    pub spheres: Spheres,
    pub aabbs: Aabbs,
    pub triangles: Triangles,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Objects {
    pub fn init(mut commands: Commands, render_state: Res<RenderState>) {
        let materials = Materials {
            data: Vec::new(),
            buffer: BufferVec::empty(&render_state.gpu_handle, "Materials Buffer"),
        };

        let spheres = Spheres {
            data: Vec::new(),
            buffer: BufferVec::empty(&render_state.gpu_handle, "Spheres Buffer"),
        };

        let aabbs = Aabbs {
            data: Vec::new(),
            buffer: BufferVec::empty(&render_state.gpu_handle, "Aabbs Buffer"),
        };

        let triangles = Triangles {
            data: Vec::new(),
            buffer: BufferVec::empty(&render_state.gpu_handle, "Triangles Buffer"),
        };

        let (bind_group_layout, bind_group) = Self::create_binding(
            &render_state.gpu_handle.device,
            &materials,
            &spheres,
            &aabbs,
            &triangles,
        );

        commands.insert_resource(Objects {
            materials,
            spheres,
            aabbs,
            triangles,
            bind_group_layout,
            bind_group,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut objects: ResMut<Objects>,
        render_state: Res<RenderState>,
        mut material_push_events: EventReader<MaterialPushEvent>,
        mut material_pop_events: EventReader<MaterialPopEvent>,
        mut sphere_push_events: EventReader<SpherePushEvent>,
        mut sphere_pop_events: EventReader<SpherePopEvent>,
        mut aabb_push_events: EventReader<AabbPushEvent>,
        mut aabb_pop_events: EventReader<AabbPopEvent>,
        mut triangle_push_events: EventReader<TrianglePushEvent>,
        mut triangle_pop_events: EventReader<TrianglePopEvent>,
    ) {
        let mut update_materials = false;
        let mut update_spheres = false;
        let mut update_aabbs = false;
        let mut update_triangles = false;

        for MaterialPushEvent(material) in material_push_events.read() {
            let mut material = *material;

            // Gamma correct on cpu side
            material.albedo = material.albedo.powf(2.2);
            material.emission = material.emission.powf(2.2);

            objects.materials.data.push(material);
            update_materials = true;
        }

        for _ in material_pop_events.read() {
            objects.materials.data.pop();
            update_materials = true;
        }

        for SpherePushEvent(sphere) in sphere_push_events.read() {
            objects.spheres.data.push(*sphere);
            update_spheres = true;
        }

        for _ in sphere_pop_events.read() {
            objects.spheres.data.pop();
            update_spheres = true;
        }

        for AabbPushEvent(aabb) in aabb_push_events.read() {
            objects.aabbs.data.push(*aabb);
            update_aabbs = true;
        }

        for _ in aabb_pop_events.read() {
            objects.aabbs.data.pop();
            update_aabbs = true;
        }

        for TrianglePushEvent(triangle) in triangle_push_events.read() {
            objects.triangles.data.push(*triangle);
            update_triangles = true;
        }

        for _ in triangle_pop_events.read() {
            objects.triangles.data.pop();
            update_triangles = true;
        }

        let mut update_bind_group = false;

        if update_materials {
            update_bind_group |= objects.materials.update_buffer();
        }

        if update_spheres {
            update_bind_group |= objects.spheres.update_buffer();
        }

        if update_aabbs {
            update_bind_group |= objects.aabbs.update_buffer();
        }

        if update_triangles {
            update_bind_group |= objects.triangles.update_buffer();
        }

        if update_bind_group {
            objects.recreate_bind_group(&render_state.gpu_handle.device);
        }
    }

    pub fn create_binding(
        device: &wgpu::Device,
        materials: &Materials,
        spheres: &Spheres,
        aabbs: &Aabbs,
        triangles: &Triangles,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        wgpu_util::binding::create_sequential_linked(
            device,
            "object_binding",
            &[
                wgpu_util::binding::BindingEntry {
                    binding_type: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    resource: materials.buffer.inner.buffer.as_entire_binding(),
                },
                wgpu_util::binding::BindingEntry {
                    binding_type: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    resource: spheres.buffer.inner.buffer.as_entire_binding(),
                },
                wgpu_util::binding::BindingEntry {
                    binding_type: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    resource: aabbs.buffer.inner.buffer.as_entire_binding(),
                },
                wgpu_util::binding::BindingEntry {
                    binding_type: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    resource: triangles.buffer.inner.buffer.as_entire_binding(),
                },
            ],
        )
    }

    pub fn recreate_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = wgpu_util::binding::create_sequential_with_layout(
            device,
            "object_binding",
            &self.bind_group_layout,
            &[
                self.materials.buffer.inner.buffer.as_entire_binding(),
                self.spheres.buffer.inner.buffer.as_entire_binding(),
                self.aabbs.buffer.inner.buffer.as_entire_binding(),
                self.triangles.buffer.inner.buffer.as_entire_binding(),
            ],
        );
    }
}

pub struct CpuGpuBuffer<T: AsStd430 + Default> {
    pub data: Vec<T>,
    pub buffer: BufferVec<T>,
}

impl<T: AsStd430 + Default> CpuGpuBuffer<T> {
    pub fn update_buffer(&mut self) -> bool {
        self.buffer.copy_from(&self.data)
    }
}

pub type Materials = CpuGpuBuffer<Material>;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaterialType {
    #[default]
    Lambertian = 0,
    Metal = 1,
    Dielectric = 2,
}

impl gpu_bytes::AsStd430 for MaterialType {
    fn as_std430(&self) -> gpu_bytes::Std430Bytes {
        (*self as u32).as_std430()
    }
}

#[derive(AsStd430, Default, Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub albedo: Vec3,
    pub roughness: f32,
    pub emission: f32,
    pub ior: f32,
    pub ty: MaterialType,
}

pub type Spheres = CpuGpuBuffer<Sphere>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_index: u32,
}

pub type Aabbs = CpuGpuBuffer<Aabb>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub material_index: u32,
}

pub type Triangles = CpuGpuBuffer<Triangle>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub material_index: u32,
}
