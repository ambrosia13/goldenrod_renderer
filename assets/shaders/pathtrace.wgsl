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
struct View_std140_0
{
    @align(16) width_0 : u32,
    @align(4) height_0 : u32,
    @align(8) aspectRatio_0 : f32,
    @align(4) frameCount_0 : u32,
};

@binding(1) @group(0) var<uniform> entryPointParams_screen_view_0 : View_std140_0;
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

@binding(0) @group(2) var entryPointParams_textures_current_0 : texture_storage_2d<rgba32float, read_write>;

@binding(1) @group(2) var entryPointParams_textures_previous_0 : texture_2d<f32>;

fn fromScreenSpace_0( screenSpacePos_0 : vec3<f32>,  matrix_0 : mat4x4<f32>) -> vec3<f32>
{
    var temp_0 : vec4<f32> = (((vec4<f32>(screenSpacePos_0 * vec3<f32>(2.0f) - vec3<f32>(1.0f), 1.0f)) * (matrix_0)));
    return temp_0.xyz / vec3<f32>(temp_0.w);
}

struct Ray_0
{
     pos_0 : vec3<f32>,
     dir_0 : vec3<f32>,
};

fn Ray_x24init_0( pos_1 : vec3<f32>,  dir_1 : vec3<f32>) -> Ray_0
{
    var _S1 : Ray_0;
    _S1.pos_0 = pos_1;
    _S1.dir_0 = dir_1;
    return _S1;
}

struct Hit_0
{
     success_0 : bool,
     position_2 : vec3<f32>,
     distance_0 : f32,
     normal_0 : vec3<f32>,
     materialIndex_3 : u32,
};

fn Hit_x24init_0() -> Hit_0
{
    var _S2 : Hit_0;
    _S2.success_0 = false;
    var _S3 : vec3<f32> = vec3<f32>(0.0f);
    _S2.position_2 = _S3;
    _S2.distance_0 = 0.0f;
    _S2.normal_0 = _S3;
    _S2.materialIndex_3 = u32(0);
    return _S2;
}

struct Sphere_0
{
     position_1 : vec3<f32>,
     radius_0 : f32,
     materialIndex_0 : u32,
};

fn Sphere_getHit_0( this_0 : Sphere_0,  ray_0 : Ray_0) -> Hit_0
{
    var hit_0 : Hit_0 = Hit_x24init_0();
    hit_0.materialIndex_3 = this_0.materialIndex_0;
    var originToCenter_0 : vec3<f32> = ray_0.pos_0 - this_0.position_1;
    var b_1 : f32 = dot(originToCenter_0, ray_0.dir_0);
    var a_1 : f32 = dot(ray_0.dir_0, ray_0.dir_0);
    var _S4 : f32 = this_0.radius_0;
    var _S5 : f32 = b_1 * b_1 - a_1 * (dot(originToCenter_0, originToCenter_0) - _S4 * _S4);
    if(_S5 >= 0.0f)
    {
        var sqrtDeterminant_0 : f32 = sqrt(_S5);
        var _S6 : f32 = - b_1;
        var _S7 : f32 = (_S6 - sqrtDeterminant_0) / a_1;
        var _S8 : f32 = (_S6 + sqrtDeterminant_0) / a_1;
        var t_0 : f32;
        if(_S7 > 0.0f)
        {
            t_0 = _S7;
        }
        else
        {
            t_0 = _S8;
        }
        if(t_0 > 0.0f)
        {
            var hitPosition_0 : vec3<f32> = ray_0.pos_0 + ray_0.dir_0 * vec3<f32>(t_0);
            var outwardNormal_0 : vec3<f32> = normalize(hitPosition_0 - this_0.position_1);
            var normal_1 : vec3<f32> = outwardNormal_0 * vec3<f32>(f32(- sign(dot(ray_0.dir_0, outwardNormal_0))));
            hit_0.success_0 = true;
            hit_0.position_2 = hitPosition_0;
            hit_0.normal_0 = normal_1;
            hit_0.distance_0 = t_0;
        }
    }
    return hit_0;
}

fn Ray_intersect_0( this_1 : Ray_0,  object_0 : Sphere_0) -> Hit_0
{
    return Sphere_getHit_0(object_0, this_1);
}

struct Aabb_0
{
     min_0 : vec3<f32>,
     max_0 : vec3<f32>,
     materialIndex_1 : u32,
};

fn Aabb_getHit_0( this_2 : Aabb_0,  ray_1 : Ray_0) -> Hit_0
{
    return Hit_x24init_0();
}

fn Ray_intersect_1( this_3 : Ray_0,  object_1 : Aabb_0) -> Hit_0
{
    return Aabb_getHit_0(object_1, this_3);
}

struct Triangle_0
{
     a_0 : vec3<f32>,
     b_0 : vec3<f32>,
     c_0 : vec3<f32>,
     materialIndex_2 : u32,
};

fn Triangle_getHit_0( this_4 : Triangle_0,  ray_2 : Ray_0) -> Hit_0
{
    return Hit_x24init_0();
}

fn Ray_intersect_2( this_5 : Ray_0,  object_2 : Triangle_0) -> Hit_0
{
    return Triangle_getHit_0(object_2, this_5);
}

fn Hit_merge_0( a_2 : Hit_0,  b_2 : Hit_0) -> Hit_0
{
    var _S9 : Hit_0 = Hit_x24init_0();
    var _S10 : bool;
    if(a_2.success_0)
    {
        _S10 = b_2.success_0;
    }
    else
    {
        _S10 = false;
    }
    var hit_1 : Hit_0;
    if(_S10)
    {
        if((a_2.distance_0) <= (b_2.distance_0))
        {
            hit_1 = a_2;
        }
        else
        {
            hit_1 = b_2;
        }
    }
    else
    {
        if(a_2.success_0)
        {
            _S10 = !b_2.success_0;
        }
        else
        {
            _S10 = false;
        }
        if(_S10)
        {
            hit_1 = a_2;
        }
        else
        {
            if(b_2.success_0)
            {
                _S10 = !a_2.success_0;
            }
            else
            {
                _S10 = false;
            }
            if(_S10)
            {
                hit_1 = b_2;
            }
            else
            {
                hit_1 = _S9;
            }
        }
    }
    return hit_1;
}

fn getCount_0() -> i32
{
    var _S11 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_spheres_0), 32);
    return i32(_S11.x);
}

fn getCount_1() -> i32
{
    var _S12 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_aabbs_0), 32);
    return i32(_S12.x);
}

fn getCount_2() -> i32
{
    var _S13 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_triangles_0), 48);
    return i32(_S13.x);
}

fn Objects_getHit_0( _S14 : Ray_0) -> Hit_0
{
    var _S15 : Hit_0 = Hit_x24init_0();
    var _S16 : i32 = getCount_0();
    var _S17 : i32 = getCount_1();
    var _S18 : i32 = getCount_2();
    var hit_2 : Hit_0 = _S15;
    var i_0 : i32 = i32(0);
    for(;;)
    {
        if(i_0 < _S16)
        {
        }
        else
        {
            break;
        }
        var _S19 : Sphere_0 = Sphere_0( entryPointParams_objects_spheres_0[i_0].position_1, entryPointParams_objects_spheres_0[i_0].radius_0, entryPointParams_objects_spheres_0[i_0].materialIndex_0 );
        var _S20 : Hit_0 = Hit_merge_0(hit_2, Ray_intersect_0(_S14, _S19));
        var i_1 : i32 = i_0 + i32(1);
        hit_2 = _S20;
        i_0 = i_1;
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S17)
        {
        }
        else
        {
            break;
        }
        var _S21 : Aabb_0 = Aabb_0( entryPointParams_objects_aabbs_0[i_0].min_0, entryPointParams_objects_aabbs_0[i_0].max_0, entryPointParams_objects_aabbs_0[i_0].materialIndex_1 );
        var _S22 : Hit_0 = Hit_merge_0(hit_2, Ray_intersect_1(_S14, _S21));
        var i_2 : i32 = i_0 + i32(1);
        hit_2 = _S22;
        i_0 = i_2;
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S18)
        {
        }
        else
        {
            break;
        }
        var _S23 : Triangle_0 = Triangle_0( entryPointParams_objects_triangles_0[i_0].a_0, entryPointParams_objects_triangles_0[i_0].b_0, entryPointParams_objects_triangles_0[i_0].c_0, entryPointParams_objects_triangles_0[i_0].materialIndex_2 );
        var _S24 : Hit_0 = Hit_merge_0(hit_2, Ray_intersect_2(_S14, _S23));
        var i_3 : i32 = i_0 + i32(1);
        hit_2 = _S24;
        i_0 = i_3;
    }
    return hit_2;
}

fn Screen_shouldAccumulate_0() -> bool
{
    var _S25 : bool;
    if(all((entryPointParams_screen_camera_0.position_0) == (entryPointParams_screen_camera_0.previousPosition_0)))
    {
        _S25 = all((entryPointParams_screen_camera_0.view_0) == (entryPointParams_screen_camera_0.previousView_0));
    }
    else
    {
        _S25 = false;
    }
    if(_S25)
    {
        _S25 = all(vec4<f32>(entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(3)][i32(0)]) == vec4<f32>(entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(3)][i32(0)]));
    }
    else
    {
        _S25 = false;
    }
    return _S25;
}

@compute
@workgroup_size(8, 8, 1)
fn compute(@builtin(global_invocation_id) globalInvocationId_0 : vec3<u32>, @builtin(local_invocation_id) localInvocationId_0 : vec3<u32>)
{
    var _S26 : u32 = globalInvocationId_0.x;
    var _S27 : bool;
    if(_S26 >= (entryPointParams_screen_view_0.width_0))
    {
        _S27 = true;
    }
    else
    {
        _S27 = (globalInvocationId_0.y) >= (entryPointParams_screen_view_0.height_0);
    }
    if(_S27)
    {
        return;
    }
    var _S28 : vec2<f32> = vec2<f32>(f32(_S26), f32(globalInvocationId_0.y)) / vec2<f32>(f32(entryPointParams_screen_view_0.width_0), f32(entryPointParams_screen_view_0.height_0));
    var texcoord_0 : vec2<f32> = _S28;
    texcoord_0[i32(1)] = 1.0f - _S28.y;
    var ray_3 : Ray_0 = Ray_x24init_0(entryPointParams_screen_camera_0.position_0, normalize(fromScreenSpace_0(vec3<f32>(texcoord_0, 1.0f), mat4x4<f32>(entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(3)])) - entryPointParams_screen_camera_0.position_0));
    var _S29 : Hit_0 = Objects_getHit_0(ray_3);
    var color_0 : vec3<f32>;
    if(_S29.success_0)
    {
        color_0 = entryPointParams_objects_materials_0[_S29.materialIndex_3].albedo_0 * vec3<f32>(max(0.0f, dot(_S29.normal_0, normalize(vec3<f32>(0.20000000298023224f, 0.40000000596046448f, -0.69999998807907104f)))));
    }
    else
    {
        color_0 = ray_3.dir_0;
    }
    var _S30 : vec2<u32> = globalInvocationId_0.xy;
    var _S31 : vec3<i32> = vec3<i32>(vec2<i32>(_S30), i32(0));
    var previousSample_0 : vec4<f32> = (textureLoad((entryPointParams_textures_previous_0), ((_S31)).xy, ((_S31)).z));
    var previousColor_0 : vec3<f32> = previousSample_0.xyz;
    var frameAge_0 : f32 = previousSample_0.w;
    var frameAge_1 : f32;
    if(Screen_shouldAccumulate_0())
    {
        var _S32 : f32 = frameAge_0 + 1.0f;
        color_0 = mix(previousColor_0, color_0, vec3<f32>((1.0f / _S32)));
        frameAge_1 = _S32;
    }
    else
    {
        frameAge_1 = 0.0f;
    }
    textureStore((entryPointParams_textures_current_0), (_S30), (vec4<f32>(color_0, frameAge_1)));
    return;
}

