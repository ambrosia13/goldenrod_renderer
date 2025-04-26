use bevy_ecs::event::Event;
use bevy_ecs::resource::Resource;
use bevy_ecs::{
    event::EventReader,
    system::{Commands, ResMut},
};
use glam::Vec3;
use gpu_bytes_derive::AsStd430;

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
        let materials = vec![Material::null()];
        let spheres = vec![Sphere::null()];
        let aabbs = vec![Aabb::null()];
        let triangles = vec![Triangle::null()];

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
