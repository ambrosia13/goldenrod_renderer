use bevy_ecs::resource::Resource;
use bevy_ecs::system::Commands;
use glam::Vec3;
use gpu_layout::{AsGpuBytes, GpuBytes};

pub mod binding;

#[derive(Resource)]
pub struct Objects {
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub aabbs: Vec<Aabb>,
    pub triangles: Vec<Triangle>,
}

impl Objects {
    pub fn init(mut commands: Commands) {
        let materials = vec![];
        let spheres = vec![];
        let aabbs = vec![];
        let triangles = vec![];

        commands.insert_resource(Objects {
            materials,
            spheres,
            aabbs,
            triangles,
        });
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

impl gpu_layout::AsGpuBytes for MaterialType {
    fn as_gpu_bytes<L: gpu_layout::GpuLayout + ?Sized>(&'_ self) -> gpu_layout::GpuBytes<'_, L> {
        let mut buf = GpuBytes::empty();

        buf.write(&(*self as u32));

        buf
    }
}

#[derive(AsGpuBytes, Default, Clone, Copy, Debug, PartialEq)]
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

#[derive(AsGpuBytes, Default, Clone, Copy, Debug)]
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

#[derive(AsGpuBytes, Default, Clone, Copy, Debug)]
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

#[derive(AsGpuBytes, Default, Clone, Copy, Debug)]
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
