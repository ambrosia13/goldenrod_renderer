# participating media renderer

- implement generalized mie theory for all particles (no special case for rayleigh) when making the participating media renderer. 
    - every parameter will be data-driven. the shader should not know specific things about the particles it renders, it should only know how to render any particle given the appropriate data
    - for phase functions, a cdf will be generated on the cpu-side, and on the gpu-side, a binary search will be performed in order to sample the phase function for a ray direction
- the object list will be entirely reworked
    - instead of treating object geometry types separately, create an geometry index integral type that contains the geometry type in the higher bits and the index in the lower bits
        - consider using a ByteAddressBuffer and lumping all object geometries into one array, essentially treating the geometry indices as type-annotated pointers
    - introduce a volume list. every object will include an inner volume index and an outer volume index. 
        - in the case of triangles, the indices will refer to the volume adjacent to the front/back faces of the triangle; we assume that triangle meshes will be closed, meaning along a view ray, we will transition into the inner volume and transition back out of the volume when intersecting the entering and exiting triangles, respectively
        - volumes will contain bindless texture indexes for lists of particles, phase functions parameters, and density profiles as needed
            - a density profile will be an enum or tagged union defining the spatial density of a volume. it can be a uniform value, based on uniform noise (in which case the shader will use a noise function to calculate the density), based on radially decaying noise (the noise will fade to zero as we approach the edge of the geometry; undefined for triangles), or it can vary based on normalized radius within the geometry (undefined for triangles) in which it contains an index into a bindless texture array
        - the volume that the viewer is currently in (which can not be easily obtained on the shader side) will be calculated on the CPU and sent to the shader: if the player is inside any object, choose the smallest object's volume index; otherwise, choose the null volume, aka the vacuum. in this calculation we will ignore triangles, planes, and any other non-volumetric geometry types 
    - introduce a geometry list that contains pure geometry information for spheres, boxes, triangles, planes, etc. (unlike the current implementation, which contains metadata such as material index)
    - a "null" material will be added, signifying no surface interaction
    - a "null" volume will be added, signifying a vacuum
    - reimplement the object type and the object list to refer to generic objects, which contain:
        - a geometry index, needed to intersect the object
        - a material index, needed to compute light interactions upon surface intersections.
        - a volume index, needed to compute light interactions when traveling inside or past the geometry
        - These three indices should be enough to accurately render any arbitrary object similar to in real life
- the atmosphere will be treated as just any two objects under this new system, with the atmosphere shell being a sphere with null material and an volume with rayleigh, mie, ozone, etc. constituents with radial density profiles. the planet shell will be a sphere with a non-null material and a null volume. there will be no special case for the atmosphere
- when tracing through volumetric objects, use a fixed-size stack to track the volumes we go in and out of, and fall back to the cpu-set default volume when the stack is empty

## pseudocode design

### Resource heap

Host: 
```rs
struct AtmosphereResourceHeap {
    textures: Vec<wgpu::Texture>,

    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}
```

Shader:
```cs
struct ResourceHeap {
    Texture1D<float>[] textures;
}
```

### Scene and objects

Host: 
```rs
struct Scene {
    base_volume_index: u32,

    objects: Vec<Object>,

    spheres: Vec<Sphere>,
    aabbs: Vec<Aabb>,
    triangles: Vec<Triangle>,
    planes: Vec<Plane>,

    materials: Vec<Material>,

    volumes: Vec<Volume>,
    volume_particles: Vec<VolumeParticle>,

    bounding_volume_hierarchy: Vec<BvhNode>,
}

struct Volume {
    particle_index: u32,
    particle_count: u32,
}

struct VolumeParticle {
    // These are all indexes into the resource heap,
    // or u32::MAX if not present or null
    scattering_cross_section_coefficients: u32,
    absorption_cross_section_coefficients: u32,
    
    // we use henyey-greenstein's G parameter for now because it's much easier
    phase_anisotropy: u32,

    density_profile: DensityProfile,
    base_density: f32, // used for all
    densities: u32, // used only in radial density, represents density lookup table by normalized radius (0 to 1)
}

enum DensityProfile {
    Uniform,
    Noise,
    FadedNoise,
    Radial,
}
```

Shader: 
```cs
struct Scene {
    ConstantBuffer<uint> baseVolumeIndex;

    StructuredBuffer<Object> objects;

    StructuredBuffer<Sphere> spheres;
    StructuredBuffer<Aabb> aabbs;
    StructuredBuffer<Triangle> triangles;
    StructuredBuffer<Plane> planes;

    StructuredBuffer<Material> materials;
    
    StructuredBuffer<Volume> volumes;
    StructuredBuffer<VolumeParticle> volume_particles;

    StructuredBuffer<BvhNode> boundingVolumeHierarchy;
}

struct Volume {
    uint particleIndex;
    uint len;
}

struct VolumeParticle {
    uint scatteringCrossSectionCoefficients;
    uint absorptionCrossSectionCoefficients;

    float phaseAnisotropy;

    DensityProfile densityProfile;
    float baseDensity;
    uint densities;
}

enum DensityProfile {
    UNIFORM,
    NOISE,
    FADED_NOISE,
    RADIAL
}
```

# infrastructure

## bounding volume hierarchy

a bounding volume hierarchy will be implemented that allows merging multiple models into one BVH for use in the shader. a simple bvh visual debugging shader will be created as an alternate display pipeline