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

pub mod binding;

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
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub aabbs: Vec<Aabb>,
    pub triangles: Vec<Triangle>,

    pub update: bool,
}

impl Objects {
    pub fn init(mut commands: Commands) {
        let materials = Vec::new();
        let spheres = Vec::new();
        let aabbs = Vec::new();
        let triangles = Vec::new();

        commands.insert_resource(Objects {
            materials,
            spheres,
            aabbs,
            triangles,
            update: false,
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
        for MaterialPushEvent(material) in material_push_events.read() {
            let mut material = *material;

            // Gamma correct on cpu side
            material.albedo = material.albedo.powf(2.2);
            material.emission = material.emission.powf(2.2);

            objects.materials.push(material);
            objects.update = true;
        }

        for _ in material_pop_events.read() {
            objects.materials.pop();
            objects.update = true;
        }

        for SpherePushEvent(sphere) in sphere_push_events.read() {
            objects.spheres.push(*sphere);
            objects.update = true;
        }

        for _ in sphere_pop_events.read() {
            objects.spheres.pop();
            objects.update = true;
        }

        for AabbPushEvent(aabb) in aabb_push_events.read() {
            objects.aabbs.push(*aabb);
            objects.update = true;
        }

        for _ in aabb_pop_events.read() {
            objects.aabbs.pop();
            objects.update = true;
        }

        for TrianglePushEvent(triangle) in triangle_push_events.read() {
            objects.triangles.push(*triangle);
            objects.update = true;
        }

        for _ in triangle_pop_events.read() {
            objects.triangles.pop();
            objects.update = true;
        }
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

impl Material {
    pub fn null() -> Self {
        Self {
            albedo: Vec3::NEG_ONE,
            ..Default::default()
        }
    }
}

pub type Spheres = CpuGpuBuffer<Sphere>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_index: u32,
}

impl Sphere {
    pub fn null() -> Self {
        Self {
            material_index: u32::MAX,
            ..Default::default()
        }
    }
}

pub type Aabbs = CpuGpuBuffer<Aabb>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub material_index: u32,
}

impl Aabb {
    pub fn null() -> Self {
        Self {
            material_index: u32::MAX,
            ..Default::default()
        }
    }
}

pub type Triangles = CpuGpuBuffer<Triangle>;

#[derive(AsStd430, Default, Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub material_index: u32,
}

impl Triangle {
    pub fn null() -> Self {
        Self {
            material_index: u32::MAX,
            ..Default::default()
        }
    }
}
