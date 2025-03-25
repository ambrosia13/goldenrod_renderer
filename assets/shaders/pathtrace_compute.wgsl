struct _MatrixStorage_float4x4_ColMajorstd140_0
{
    @align(16) data_0 : array<vec4<f32>, i32(4)>,
};

struct Camera_std140_0
{
    @align(16) viewProjectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) viewMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) projectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) inverseViewProjectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) inverseViewMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) inverseProjectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) previousViewProjectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) previousViewMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) previousProjectionMatrix_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) position_0 : vec3<f32>,
    @align(16) previousPosition_0 : vec3<f32>,
    @align(16) view_0 : vec3<f32>,
    @align(16) previousView_0 : vec3<f32>,
    @align(16) right_0 : vec3<f32>,
    @align(16) up_0 : vec3<f32>,
};

@binding(0) @group(0) var<uniform> entryPointParams_screen_camera_0 : Camera_std140_0;
struct Material_std430_0
{
    @align(16) albedo_0 : vec3<f32>,
    @align(4) roughness_0 : f32,
    @align(16) emission_0 : vec3<f32>,
    @align(4) ior_0 : f32,
    @align(16) type_0 : u32,
};

@binding(0) @group(1) var<storage, read> entryPointParams_objects_materials_0 : array<Material_std430_0>;

struct Sphere_std430_0
{
    @align(16) position_1 : vec3<f32>,
    @align(4) radius_0 : f32,
    @align(16) materialIndex_0 : u32,
};

@binding(1) @group(1) var<storage, read> entryPointParams_objects_spheres_0 : array<Sphere_std430_0>;

struct Aabb_std430_0
{
    @align(16) min_0 : vec3<f32>,
    @align(16) max_0 : vec3<f32>,
    @align(4) materialIndex_1 : u32,
};

@binding(2) @group(1) var<storage, read> entryPointParams_objects_aabbs_0 : array<Aabb_std430_0>;

struct Triangle_std430_0
{
    @align(16) a_0 : vec3<f32>,
    @align(16) b_0 : vec3<f32>,
    @align(16) c_0 : vec3<f32>,
    @align(4) materialIndex_2 : u32,
};

@binding(3) @group(1) var<storage, read> entryPointParams_objects_triangles_0 : array<Triangle_std430_0>;

@binding(0) @group(2) var<uniform> entryPointParams_test_m_0 : f32;
@binding(1) @group(2) var<uniform> entryPointParams_test_t_0 : f32;
var entryPointParams_colorTexture_0 : texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn compute(@builtin(global_invocation_id) globalInvocationIndex_0 : vec3<u32>, @builtin(local_invocation_id) localInvocationIndex_0 : vec3<u32>)
{
    const _S1 : vec2<u32> = vec2<u32>(u32(0), u32(0));
    textureStore((entryPointParams_colorTexture_0), (_S1), (entryPointParams_screen_camera_0.up_0.xyzx * vec4<f32>(entryPointParams_test_m_0)));
    textureStore((entryPointParams_colorTexture_0), (_S1), (entryPointParams_objects_spheres_0[i32(0)].position_1.xyzx));
    textureStore((entryPointParams_colorTexture_0), (_S1), (entryPointParams_objects_materials_0[i32(0)].albedo_0.xyzx));
    textureStore((entryPointParams_colorTexture_0), (_S1), (entryPointParams_objects_triangles_0[i32(0)].a_0.xyzx));
    textureStore((entryPointParams_colorTexture_0), (_S1), (entryPointParams_objects_aabbs_0[i32(0)].min_0.xyzx + vec4<f32>(entryPointParams_test_t_0)));
    return;
}

