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
    @align(16) emission_0 : f32,
    @align(4) ior_0 : f32,
    @align(8) type_0 : u32,
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

fn Camera_fromScreenSpace_0( screenSpacePos_0 : vec3<f32>,  matrix_0 : mat4x4<f32>) -> vec3<f32>
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

fn Sphere_isUnhittable_0( this_0 : Sphere_0) -> bool
{
    return (this_0.radius_0) == 0.0f;
}

fn Sphere_getHit_0( this_1 : Sphere_0,  ray_0 : Ray_0) -> Hit_0
{
    var hit_0 : Hit_0 = Hit_x24init_0();
    hit_0.materialIndex_3 = this_1.materialIndex_0;
    var originToCenter_0 : vec3<f32> = ray_0.pos_0 - this_1.position_1;
    var b_1 : f32 = dot(originToCenter_0, ray_0.dir_0);
    var a_1 : f32 = dot(ray_0.dir_0, ray_0.dir_0);
    var _S4 : f32 = this_1.radius_0;
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
            var outwardNormal_0 : vec3<f32> = normalize(hitPosition_0 - this_1.position_1);
            var normal_1 : vec3<f32> = outwardNormal_0 * vec3<f32>(f32(- sign(dot(ray_0.dir_0, outwardNormal_0))));
            hit_0.success_0 = true;
            hit_0.position_2 = hitPosition_0;
            hit_0.normal_0 = normal_1;
            hit_0.distance_0 = t_0;
        }
    }
    return hit_0;
}

fn Ray_intersect_0( this_2 : Ray_0,  object_0 : Sphere_0) -> Hit_0
{
    return Sphere_getHit_0(object_0, this_2);
}

struct Aabb_0
{
     min_0 : vec3<f32>,
     max_0 : vec3<f32>,
     materialIndex_1 : u32,
};

fn Aabb_getHit_0( this_3 : Aabb_0,  ray_1 : Ray_0) -> Hit_0
{
    return Hit_x24init_0();
}

fn Ray_intersect_1( this_4 : Ray_0,  object_1 : Aabb_0) -> Hit_0
{
    return Aabb_getHit_0(object_1, this_4);
}

struct Triangle_0
{
     a_0 : vec3<f32>,
     b_0 : vec3<f32>,
     c_0 : vec3<f32>,
     materialIndex_2 : u32,
};

fn Triangle_getHit_0( this_5 : Triangle_0,  ray_2 : Ray_0) -> Hit_0
{
    return Hit_x24init_0();
}

fn Ray_intersect_2( this_6 : Ray_0,  object_2 : Triangle_0) -> Hit_0
{
    return Triangle_getHit_0(object_2, this_6);
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

fn Aabb_isUnhittable_0( this_7 : Aabb_0) -> bool
{
    return all((this_7.max_0) > (this_7.min_0));
}

fn Triangle_isUnhittable_0( this_8 : Triangle_0) -> bool
{
    var _S11 : bool;
    if(all((this_8.a_0) == (this_8.b_0)))
    {
        _S11 = true;
    }
    else
    {
        _S11 = all((this_8.b_0) == (this_8.c_0));
    }
    if(_S11)
    {
        _S11 = true;
    }
    else
    {
        _S11 = all((this_8.a_0) == (this_8.c_0));
    }
    return _S11;
}

struct Material_0
{
     albedo_0 : vec3<f32>,
     roughness_0 : f32,
     emission_0 : f32,
     ior_0 : f32,
     type_0 : u32,
};

fn Material_getAlbedo_0( this_9 : Material_0) -> vec3<f32>
{
    return this_9.albedo_0 * vec3<f32>(step(this_9.emission_0, 0.00009999999747379f));
}

fn pcg_0( seed_0 : ptr<function, u32>)
{
    var state_0 : u32 = (*seed_0) * u32(747796405) + u32(2891336453);
    var word_0 : u32 = ((((state_0 >> ((((state_0 >> (u32(28)))) + u32(4))))) ^ (state_0))) * u32(277803737);
    (*seed_0) = (((word_0 >> (u32(22)))) ^ (word_0));
    return;
}

struct Random_0
{
     state_1 : u32,
};

fn Random_getUint_0( this_10 : ptr<function, Random_0>) -> u32
{
    var _S12 : u32 = (*this_10).state_1;
    pcg_0(&(_S12));
    (*this_10).state_1 = _S12;
    return _S12;
}

fn Random_getFloat_0( this_11 : ptr<function, Random_0>) -> f32
{
    var _S13 : u32 = Random_getUint_0(&((*this_11)));
    return f32(_S13) / 4.294967296e+09f;
}

fn getTbnMatrix_0( normal_2 : vec3<f32>) -> mat3x3<f32>
{
    var tangent_0 : vec3<f32> = normalize(cross(normalize(vec3<f32>(1.0f, 0.5f, 0.25f)), normal_2));
    return mat3x3<f32>(tangent_0, normalize(cross(normal_2, tangent_0)), normal_2);
}

fn ggxNormal_0( normal_3 : vec3<f32>,  roughness_1 : f32,  random_0 : ptr<function, Random_0>) -> vec3<f32>
{
    var r1_0 : f32 = Random_getFloat_0(&((*random_0)));
    var r2_0 : f32 = Random_getFloat_0(&((*random_0)));
    var a_3 : f32 = roughness_1 * roughness_1;
    var phi_0 : f32 = 6.28318548202514648f * r1_0;
    var cosTheta_0 : f32 = sqrt((1.0f - r2_0) / (1.0f + (a_3 * a_3 - 1.0f) * r2_0));
    var sinTheta_0 : f32 = sqrt(1.0f - cosTheta_0 * cosTheta_0);
    return (((getTbnMatrix_0(normal_3)) * (vec3<f32>(sinTheta_0 * cos(phi_0), sinTheta_0 * sin(phi_0), cosTheta_0))));
}

fn Material_evaluateBrdf_0( this_12 : Material_0,  hit_2 : Hit_0,  random_1 : ptr<function, Random_0>,  nextRay_0 : ptr<function, Ray_0>) -> vec3<f32>
{
    if((this_12.type_0) == u32(0))
    {
        var brdf_0 : vec3<f32> = Material_getAlbedo_0(this_12) / vec3<f32>(3.14159274101257324f);
        var _S14 : vec3<f32> = hit_2.position_2 + hit_2.normal_0 * vec3<f32>(0.00009999999747379f);
        var _S15 : vec3<f32> = ggxNormal_0(hit_2.normal_0, 1.0f, &((*random_1)));
        (*nextRay_0) = Ray_x24init_0(_S14, _S15);
        return brdf_0;
    }
    else
    {
        var _S16 : vec3<f32> = vec3<f32>(0.0f);
        (*nextRay_0) = Ray_x24init_0(_S16, _S16);
        return _S16;
    }
}

fn Material_getEmission_0( this_13 : Material_0) -> vec3<f32>
{
    return this_13.albedo_0 * vec3<f32>(this_13.emission_0);
}

fn Camera_screenToScene_0( _S17 : vec3<f32>) -> vec3<f32>
{
    return Camera_fromScreenSpace_0(_S17, mat4x4<f32>(entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(3)])) - entryPointParams_screen_camera_0.position_0;
}

fn Random_x24init_0( _S18 : vec2<u32>) -> Random_0
{
    var _S19 : Random_0;
    _S19.state_1 = entryPointParams_screen_view_0.width_0 * entryPointParams_screen_view_0.height_0 * (entryPointParams_screen_view_0.frameCount_0 + u32(1)) * (_S18.x + _S18.y * entryPointParams_screen_view_0.width_0);
    return _S19;
}

fn getCount_0() -> i32
{
    var _S20 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_spheres_0), 32);
    return i32(_S20.x);
}

fn getCount_1() -> i32
{
    var _S21 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_aabbs_0), 32);
    return i32(_S21.x);
}

fn getCount_2() -> i32
{
    var _S22 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_triangles_0), 48);
    return i32(_S22.x);
}

fn Objects_getHit_0( _S23 : Ray_0) -> Hit_0
{
    var _S24 : Hit_0 = Hit_x24init_0();
    var _S25 : i32 = getCount_0();
    var _S26 : i32 = getCount_1();
    var _S27 : i32 = getCount_2();
    var hit_3 : Hit_0 = _S24;
    var i_0 : i32 = i32(0);
    for(;;)
    {
        if(i_0 < _S25)
        {
        }
        else
        {
            break;
        }
        var _S28 : Sphere_0 = Sphere_0( entryPointParams_objects_spheres_0[i_0].position_1, entryPointParams_objects_spheres_0[i_0].radius_0, entryPointParams_objects_spheres_0[i_0].materialIndex_0 );
        if(Sphere_isUnhittable_0(_S28))
        {
            i_0 = i_0 + i32(1);
            continue;
        }
        var _S29 : Sphere_0 = Sphere_0( entryPointParams_objects_spheres_0[i_0].position_1, entryPointParams_objects_spheres_0[i_0].radius_0, entryPointParams_objects_spheres_0[i_0].materialIndex_0 );
        hit_3 = Hit_merge_0(hit_3, Ray_intersect_0(_S23, _S29));
        i_0 = i_0 + i32(1);
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S26)
        {
        }
        else
        {
            break;
        }
        var _S30 : Aabb_0 = Aabb_0( entryPointParams_objects_aabbs_0[i_0].min_0, entryPointParams_objects_aabbs_0[i_0].max_0, entryPointParams_objects_aabbs_0[i_0].materialIndex_1 );
        if(Aabb_isUnhittable_0(_S30))
        {
            i_0 = i_0 + i32(1);
            continue;
        }
        var _S31 : Aabb_0 = Aabb_0( entryPointParams_objects_aabbs_0[i_0].min_0, entryPointParams_objects_aabbs_0[i_0].max_0, entryPointParams_objects_aabbs_0[i_0].materialIndex_1 );
        hit_3 = Hit_merge_0(hit_3, Ray_intersect_1(_S23, _S31));
        i_0 = i_0 + i32(1);
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S27)
        {
        }
        else
        {
            break;
        }
        var _S32 : Triangle_0 = Triangle_0( entryPointParams_objects_triangles_0[i_0].a_0, entryPointParams_objects_triangles_0[i_0].b_0, entryPointParams_objects_triangles_0[i_0].c_0, entryPointParams_objects_triangles_0[i_0].materialIndex_2 );
        if(Triangle_isUnhittable_0(_S32))
        {
            i_0 = i_0 + i32(1);
            continue;
        }
        var _S33 : Triangle_0 = Triangle_0( entryPointParams_objects_triangles_0[i_0].a_0, entryPointParams_objects_triangles_0[i_0].b_0, entryPointParams_objects_triangles_0[i_0].c_0, entryPointParams_objects_triangles_0[i_0].materialIndex_2 );
        hit_3 = Hit_merge_0(hit_3, Ray_intersect_2(_S23, _S33));
        i_0 = i_0 + i32(1);
    }
    return hit_3;
}

fn pathtrace_0( _S34 : Ray_0,  _S35 : ptr<function, Random_0>) -> vec3<f32>
{
    var _S36 : vec3<f32> = vec3<f32>(1.0f);
    var _S37 : vec3<f32> = vec3<f32>(0.0f);
    var _S38 : Ray_0 = _S34;
    var i_1 : i32 = i32(0);
    var throughput_0 : vec3<f32> = _S36;
    var radiance_0 : vec3<f32> = _S37;
    var _S39 : vec3<f32> = vec3<f32>(3.14159274101257324f);
    for(;;)
    {
        if(i_1 < i32(5))
        {
        }
        else
        {
            break;
        }
        var _S40 : Hit_0 = Objects_getHit_0(_S38);
        if(!_S40.success_0)
        {
            radiance_0 = radiance_0 + throughput_0 * _S37;
            break;
        }
        var _S41 : Material_0 = Material_0( entryPointParams_objects_materials_0[_S40.materialIndex_3].albedo_0, entryPointParams_objects_materials_0[_S40.materialIndex_3].roughness_0, entryPointParams_objects_materials_0[_S40.materialIndex_3].emission_0, entryPointParams_objects_materials_0[_S40.materialIndex_3].ior_0, entryPointParams_objects_materials_0[_S40.materialIndex_3].type_0 );
        var nextRay_1 : Ray_0;
        var brdf_1 : vec3<f32> = Material_evaluateBrdf_0(_S41, _S40, &((*_S35)), &(nextRay_1));
        var radiance_1 : vec3<f32> = radiance_0 + throughput_0 * Material_getEmission_0(_S41);
        var throughput_1 : vec3<f32> = throughput_0 * (brdf_1 / _S39);
        var _S42 : Ray_0 = nextRay_1;
        var i_2 : i32 = i_1 + i32(1);
        _S38 = _S42;
        i_1 = i_2;
        throughput_0 = throughput_1;
        radiance_0 = radiance_1;
    }
    return radiance_0;
}

fn Screen_shouldAccumulate_0() -> bool
{
    var _S43 : bool;
    if(all((entryPointParams_screen_camera_0.position_0) == (entryPointParams_screen_camera_0.previousPosition_0)))
    {
        _S43 = all((entryPointParams_screen_camera_0.view_0) == (entryPointParams_screen_camera_0.previousView_0));
    }
    else
    {
        _S43 = false;
    }
    if(_S43)
    {
        _S43 = all(vec4<f32>(entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(3)][i32(0)]) == vec4<f32>(entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(3)][i32(0)]));
    }
    else
    {
        _S43 = false;
    }
    return _S43;
}

@compute
@workgroup_size(8, 8, 1)
fn compute(@builtin(global_invocation_id) globalInvocationId_0 : vec3<u32>, @builtin(local_invocation_id) localInvocationId_0 : vec3<u32>)
{
    var _S44 : u32 = globalInvocationId_0.x;
    var _S45 : bool;
    if(_S44 >= (entryPointParams_screen_view_0.width_0))
    {
        _S45 = true;
    }
    else
    {
        _S45 = (globalInvocationId_0.y) >= (entryPointParams_screen_view_0.height_0);
    }
    if(_S45)
    {
        return;
    }
    var _S46 : vec2<f32> = vec2<f32>(f32(_S44), f32(globalInvocationId_0.y)) / vec2<f32>(f32(entryPointParams_screen_view_0.width_0), f32(entryPointParams_screen_view_0.height_0));
    var texcoord_0 : vec2<f32> = _S46;
    texcoord_0[i32(1)] = 1.0f - _S46.y;
    var viewDir_0 : vec3<f32> = normalize(Camera_screenToScene_0(vec3<f32>(texcoord_0, 1.0f)));
    var _S47 : vec2<u32> = globalInvocationId_0.xy;
    var random_2 : Random_0 = Random_x24init_0(_S47);
    var _S48 : vec3<f32> = pathtrace_0(Ray_x24init_0(entryPointParams_screen_camera_0.position_0, viewDir_0), &(random_2));
    var _S49 : vec3<i32> = vec3<i32>(vec2<i32>(_S47), i32(0));
    var previousSample_0 : vec4<f32> = (textureLoad((entryPointParams_textures_previous_0), ((_S49)).xy, ((_S49)).z));
    var previousColor_0 : vec3<f32> = previousSample_0.xyz;
    var frameAge_0 : f32 = previousSample_0.w;
    var color_0 : vec3<f32>;
    var frameAge_1 : f32;
    if(Screen_shouldAccumulate_0())
    {
        var _S50 : f32 = frameAge_0 + 1.0f;
        color_0 = mix(previousColor_0, _S48, vec3<f32>((1.0f / _S50)));
        frameAge_1 = _S50;
    }
    else
    {
        color_0 = _S48;
        frameAge_1 = 0.0f;
    }
    textureStore((entryPointParams_textures_current_0), (_S47), (vec4<f32>(color_0, frameAge_1)));
    return;
}

