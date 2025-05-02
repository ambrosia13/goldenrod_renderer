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
    @align(16) boundsMin_0 : vec3<f32>,
    @align(16) boundsMax_0 : vec3<f32>,
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

@binding(0) @group(3) var entryPointParams_spectrum_wavelengthToXyzLut_0 : texture_storage_1d<rgba32float, read_write>;

@binding(1) @group(3) var entryPointParams_spectrum_rgbToSpectralIntensityLut_0 : texture_storage_1d<rgba32float, read_write>;

struct Random_0
{
     state_0 : u32,
};

fn Random_x24init_0( seed_0 : u32) -> Random_0
{
    var _S1 : Random_0;
    _S1.state_0 = seed_0;
    return _S1;
}

fn getTaaOffset_0( frame_0 : u32) -> vec2<f32>
{
    var taaOffsets_0 : array<vec2<f32>, i32(8)> = array<vec2<f32>, i32(8)>( vec2<f32>(0.125f, -0.375f), vec2<f32>(-0.125f, 0.375f), vec2<f32>(0.625f, 0.125f), vec2<f32>(0.375f, -0.625f), vec2<f32>(-0.625f, 0.625f), vec2<f32>(-0.875f, -0.125f), vec2<f32>(0.375f, -0.875f), vec2<f32>(0.875f, 0.875f) );
    return taaOffsets_0[frame_0 % u32(8)];
}

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
    var _S2 : Ray_0;
    _S2.pos_0 = pos_1;
    _S2.dir_0 = dir_1;
    return _S2;
}

fn pcg_0( seed_1 : ptr<function, u32>)
{
    var state_1 : u32 = (*seed_1) * u32(747796405) + u32(2891336453);
    var word_0 : u32 = ((((state_1 >> ((((state_1 >> (u32(28)))) + u32(4))))) ^ (state_1))) * u32(277803737);
    (*seed_1) = (((word_0 >> (u32(22)))) ^ (word_0));
    return;
}

fn Random_getUint_0( this_0 : ptr<function, Random_0>) -> u32
{
    var _S3 : u32 = (*this_0).state_0;
    pcg_0(&(_S3));
    (*this_0).state_0 = _S3;
    return _S3;
}

fn Random_getFloat_0( this_1 : ptr<function, Random_0>) -> f32
{
    var _S4 : u32 = Random_getUint_0(&((*this_1)));
    return f32(_S4) / 4.294967296e+09f;
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
    var _S5 : Hit_0;
    _S5.success_0 = false;
    var _S6 : vec3<f32> = vec3<f32>(0.0f);
    _S5.position_2 = _S6;
    _S5.distance_0 = 0.0f;
    _S5.normal_0 = _S6;
    _S5.materialIndex_3 = u32(0);
    return _S5;
}

struct Sphere_0
{
     position_1 : vec3<f32>,
     radius_0 : f32,
     materialIndex_0 : u32,
};

fn Sphere_isUnhittable_0( this_2 : Sphere_0) -> bool
{
    var _S7 : bool;
    if((this_2.radius_0) <= 0.0f)
    {
        _S7 = true;
    }
    else
    {
        _S7 = (this_2.materialIndex_0) == u32(4294967295);
    }
    return _S7;
}

fn Sphere_getHit_0( this_3 : Sphere_0,  ray_0 : Ray_0) -> Hit_0
{
    var hit_0 : Hit_0 = Hit_x24init_0();
    hit_0.materialIndex_3 = this_3.materialIndex_0;
    var originToCenter_0 : vec3<f32> = ray_0.pos_0 - this_3.position_1;
    var b_1 : f32 = dot(originToCenter_0, ray_0.dir_0);
    var a_1 : f32 = dot(ray_0.dir_0, ray_0.dir_0);
    var _S8 : f32 = this_3.radius_0;
    var _S9 : f32 = b_1 * b_1 - a_1 * (dot(originToCenter_0, originToCenter_0) - _S8 * _S8);
    if(_S9 >= 0.0f)
    {
        var sqrtDeterminant_0 : f32 = sqrt(_S9);
        var _S10 : f32 = - b_1;
        var _S11 : f32 = (_S10 - sqrtDeterminant_0) / a_1;
        var _S12 : f32 = (_S10 + sqrtDeterminant_0) / a_1;
        var t_0 : f32;
        if(_S11 > 0.0f)
        {
            t_0 = _S11;
        }
        else
        {
            t_0 = _S12;
        }
        if(t_0 > 0.0f)
        {
            var hitPosition_0 : vec3<f32> = ray_0.pos_0 + ray_0.dir_0 * vec3<f32>(t_0);
            var outwardNormal_0 : vec3<f32> = normalize(hitPosition_0 - this_3.position_1);
            var normal_1 : vec3<f32> = outwardNormal_0 * vec3<f32>(f32(- sign(dot(ray_0.dir_0, outwardNormal_0))));
            hit_0.success_0 = true;
            hit_0.position_2 = hitPosition_0;
            hit_0.normal_0 = normal_1;
            hit_0.distance_0 = t_0;
        }
    }
    return hit_0;
}

fn Ray_intersect_0( this_4 : Ray_0,  object_0 : Sphere_0) -> Hit_0
{
    return Sphere_getHit_0(object_0, this_4);
}

struct Aabb_0
{
     boundsMin_0 : vec3<f32>,
     boundsMax_0 : vec3<f32>,
     materialIndex_1 : u32,
};

fn max3_0( x_0 : f32,  y_0 : f32,  z_0 : f32) -> f32
{
    return max(x_0, max(y_0, z_0));
}

fn min3_0( x_1 : f32,  y_1 : f32,  z_1 : f32) -> f32
{
    return min(x_1, min(y_1, z_1));
}

fn Aabb_getHit_0( this_5 : Aabb_0,  ray_1 : Ray_0) -> Hit_0
{
    var hit_1 : Hit_0 = Hit_x24init_0();
    hit_1.materialIndex_3 = this_5.materialIndex_1;
    var tMin_0 : vec3<f32> = (this_5.boundsMin_0 - ray_1.pos_0) / ray_1.dir_0;
    var tMax_0 : vec3<f32> = (this_5.boundsMax_0 - ray_1.pos_0) / ray_1.dir_0;
    var t1_0 : vec3<f32> = min(tMin_0, tMax_0);
    var t2_0 : vec3<f32> = max(tMin_0, tMax_0);
    var tNear_0 : f32 = max3_0(t1_0.x, t1_0.y, t1_0.z);
    var tFar_0 : f32 = min3_0(t2_0.x, t2_0.y, t2_0.z);
    if(!all((clamp(ray_1.pos_0, this_5.boundsMin_0, this_5.boundsMax_0)) == (ray_1.pos_0)))
    {
        var _S13 : bool;
        if(tNear_0 >= tFar_0)
        {
            _S13 = true;
        }
        else
        {
            _S13 = tFar_0 <= 0.0f;
        }
        hit_1.success_0 = !_S13;
        hit_1.distance_0 = tNear_0;
        var eq_0 : vec3<bool> = t1_0 == vec3<f32>(tNear_0);
        hit_1.normal_0 = vec3<f32>(f32(eq_0.x), f32(eq_0.y), f32(eq_0.z)) * vec3<f32>(sign(- ray_1.dir_0));
    }
    else
    {
        hit_1.success_0 = true;
        hit_1.distance_0 = tFar_0;
        var eq_1 : vec3<bool> = t2_0 == vec3<f32>(tFar_0);
        hit_1.normal_0 = vec3<f32>(f32(eq_1.x), f32(eq_1.y), f32(eq_1.z)) * vec3<f32>(sign(- ray_1.dir_0));
    }
    hit_1.position_2 = ray_1.pos_0 + ray_1.dir_0 * vec3<f32>(hit_1.distance_0);
    return hit_1;
}

fn Ray_intersect_1( this_6 : Ray_0,  object_1 : Aabb_0) -> Hit_0
{
    return Aabb_getHit_0(object_1, this_6);
}

struct Triangle_0
{
     a_0 : vec3<f32>,
     b_0 : vec3<f32>,
     c_0 : vec3<f32>,
     materialIndex_2 : u32,
};

fn Triangle_getHit_0( this_7 : Triangle_0,  ray_2 : Ray_0) -> Hit_0
{
    return Hit_x24init_0();
}

fn Ray_intersect_2( this_8 : Ray_0,  object_2 : Triangle_0) -> Hit_0
{
    return Triangle_getHit_0(object_2, this_8);
}

fn Hit_merge_0( a_2 : Hit_0,  b_2 : Hit_0) -> Hit_0
{
    var _S14 : Hit_0 = Hit_x24init_0();
    var _S15 : bool;
    if(a_2.success_0)
    {
        _S15 = b_2.success_0;
    }
    else
    {
        _S15 = false;
    }
    var hit_2 : Hit_0;
    if(_S15)
    {
        if((a_2.distance_0) <= (b_2.distance_0))
        {
            hit_2 = a_2;
        }
        else
        {
            hit_2 = b_2;
        }
    }
    else
    {
        if(a_2.success_0)
        {
            _S15 = !b_2.success_0;
        }
        else
        {
            _S15 = false;
        }
        if(_S15)
        {
            hit_2 = a_2;
        }
        else
        {
            if(b_2.success_0)
            {
                _S15 = !a_2.success_0;
            }
            else
            {
                _S15 = false;
            }
            if(_S15)
            {
                hit_2 = b_2;
            }
            else
            {
                hit_2 = _S14;
            }
        }
    }
    return hit_2;
}

fn Aabb_isUnhittable_0( this_9 : Aabb_0) -> bool
{
    var _S16 : bool;
    if(any((this_9.boundsMax_0) <= (this_9.boundsMin_0)))
    {
        _S16 = true;
    }
    else
    {
        _S16 = (this_9.materialIndex_1) == u32(4294967295);
    }
    return _S16;
}

fn Triangle_isUnhittable_0( this_10 : Triangle_0) -> bool
{
    var _S17 : bool;
    if(all((this_10.a_0) == (this_10.b_0)))
    {
        _S17 = true;
    }
    else
    {
        _S17 = all((this_10.b_0) == (this_10.c_0));
    }
    if(_S17)
    {
        _S17 = true;
    }
    else
    {
        _S17 = all((this_10.a_0) == (this_10.c_0));
    }
    if(_S17)
    {
        _S17 = true;
    }
    else
    {
        _S17 = (this_10.materialIndex_2) == u32(4294967295);
    }
    return _S17;
}

fn sky_0( ray_3 : Ray_0,  random_0 : ptr<function, Random_0>) -> vec3<f32>
{
    return vec3<f32>(0.5f);
}

fn rcp_0( x_2 : f32) -> f32
{
    return 1.0f / x_2;
}

struct Material_0
{
     albedo_0 : vec3<f32>,
     roughness_0 : f32,
     emission_0 : f32,
     ior_0 : f32,
     type_0 : u32,
};

fn Material_getAlbedo_0( this_11 : Material_0) -> vec3<f32>
{
    return this_11.albedo_0 * vec3<f32>(step(this_11.emission_0, 0.00009999999747379f));
}

fn Random_getUnitVector_0( this_12 : ptr<function, Random_0>) -> vec3<f32>
{
    var _S18 : f32 = Random_getFloat_0(&((*this_12)));
    var _S19 : f32 = Random_getFloat_0(&((*this_12)));
    var xy_0 : vec2<f32> = vec2<f32>(_S18, _S19);
    xy_0[i32(0)] = xy_0[i32(0)] * 6.28318548202514648f;
    xy_0[i32(1)] = 2.0f * xy_0.y - 1.0f;
    return vec3<f32>(vec2<f32>(sin(xy_0.x), cos(xy_0.x)) * vec2<f32>(sqrt(1.0f - xy_0.y * xy_0.y)), xy_0.y);
}

fn Random_getCosineVector_0( this_13 : ptr<function, Random_0>,  normal_2 : vec3<f32>) -> vec3<f32>
{
    var _S20 : vec3<f32> = Random_getUnitVector_0(&((*this_13)));
    return normalize(normal_2 + _S20);
}

fn Material_getEmission_0( this_14 : Material_0) -> vec3<f32>
{
    return this_14.albedo_0 * vec3<f32>(this_14.emission_0);
}

fn xyzToRgb_0( xyz_0 : vec3<f32>) -> vec3<f32>
{
    return (((mat3x3<f32>(3.24045419692993164f, -0.96926599740982056f, 0.05564339831471443f, -1.53713846206665039f, 1.87601077556610107f, -0.20402589440345764f, -0.49853140115737915f, 0.04155600070953369f, 1.05722522735595703f)) * (xyz_0)));
}

fn Screen_shouldAccumulate_0() -> bool
{
    var _S21 : bool;
    if(all((entryPointParams_screen_camera_0.position_0) == (entryPointParams_screen_camera_0.previousPosition_0)))
    {
        _S21 = all((entryPointParams_screen_camera_0.view_0) == (entryPointParams_screen_camera_0.previousView_0));
    }
    else
    {
        _S21 = false;
    }
    if(_S21)
    {
        _S21 = all(vec4<f32>(entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.projectionMatrix_0.data_0[i32(3)][i32(0)]) == vec4<f32>(entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.previousProjectionMatrix_0.data_0[i32(3)][i32(0)]));
    }
    else
    {
        _S21 = false;
    }
    return _S21;
}

fn Camera_screenToScene_0( _S22 : vec3<f32>) -> vec3<f32>
{
    return Camera_fromScreenSpace_0(_S22, mat4x4<f32>(entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(0)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(1)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(2)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(0)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(1)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(2)][i32(3)], entryPointParams_screen_camera_0.inverseViewProjectionMatrix_0.data_0[i32(3)][i32(3)])) - entryPointParams_screen_camera_0.position_0;
}

fn Camera_viewDir_0( _S23 : vec2<f32>) -> vec3<f32>
{
    return normalize(Camera_screenToScene_0(vec3<f32>(_S23, 1.0f)));
}

fn Spectrum_generateWavelength_0( _S24 : ptr<function, Random_0>) -> f32
{
    var _S25 : f32 = Random_getFloat_0(&((*_S24)));
    return _S25 * 400.0f + 380.0f;
}

fn getCount_0() -> i32
{
    var _S26 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_spheres_0), 32);
    return i32(_S26.x);
}

fn getCount_1() -> i32
{
    var _S27 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_aabbs_0), 32);
    return i32(_S27.x);
}

fn getCount_2() -> i32
{
    var _S28 : vec2<u32> = vec2<u32>(arrayLength(&entryPointParams_objects_triangles_0), 48);
    return i32(_S28.x);
}

fn Objects_getHit_0( _S29 : Ray_0) -> Hit_0
{
    var _S30 : Hit_0 = Hit_x24init_0();
    var _S31 : i32 = getCount_0();
    var _S32 : i32 = getCount_1();
    var _S33 : i32 = getCount_2();
    var hit_3 : Hit_0 = _S30;
    var i_0 : i32 = i32(0);
    for(;;)
    {
        if(i_0 < _S31)
        {
        }
        else
        {
            break;
        }
        var _S34 : Sphere_0 = Sphere_0( entryPointParams_objects_spheres_0[i_0].position_1, entryPointParams_objects_spheres_0[i_0].radius_0, entryPointParams_objects_spheres_0[i_0].materialIndex_0 );
        if(Sphere_isUnhittable_0(_S34))
        {
            break;
        }
        var _S35 : Sphere_0 = Sphere_0( entryPointParams_objects_spheres_0[i_0].position_1, entryPointParams_objects_spheres_0[i_0].radius_0, entryPointParams_objects_spheres_0[i_0].materialIndex_0 );
        var _S36 : Hit_0 = Hit_merge_0(hit_3, Ray_intersect_0(_S29, _S35));
        var i_1 : i32 = i_0 + i32(1);
        hit_3 = _S36;
        i_0 = i_1;
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S32)
        {
        }
        else
        {
            break;
        }
        var _S37 : Aabb_0 = Aabb_0( entryPointParams_objects_aabbs_0[i_0].boundsMin_0, entryPointParams_objects_aabbs_0[i_0].boundsMax_0, entryPointParams_objects_aabbs_0[i_0].materialIndex_1 );
        if(Aabb_isUnhittable_0(_S37))
        {
            break;
        }
        var _S38 : Aabb_0 = Aabb_0( entryPointParams_objects_aabbs_0[i_0].boundsMin_0, entryPointParams_objects_aabbs_0[i_0].boundsMax_0, entryPointParams_objects_aabbs_0[i_0].materialIndex_1 );
        var _S39 : Hit_0 = Hit_merge_0(hit_3, Ray_intersect_1(_S29, _S38));
        var i_2 : i32 = i_0 + i32(1);
        hit_3 = _S39;
        i_0 = i_2;
    }
    i_0 = i32(0);
    for(;;)
    {
        if(i_0 < _S33)
        {
        }
        else
        {
            break;
        }
        var _S40 : Triangle_0 = Triangle_0( entryPointParams_objects_triangles_0[i_0].a_0, entryPointParams_objects_triangles_0[i_0].b_0, entryPointParams_objects_triangles_0[i_0].c_0, entryPointParams_objects_triangles_0[i_0].materialIndex_2 );
        if(Triangle_isUnhittable_0(_S40))
        {
            break;
        }
        var _S41 : Triangle_0 = Triangle_0( entryPointParams_objects_triangles_0[i_0].a_0, entryPointParams_objects_triangles_0[i_0].b_0, entryPointParams_objects_triangles_0[i_0].c_0, entryPointParams_objects_triangles_0[i_0].materialIndex_2 );
        var _S42 : Hit_0 = Hit_merge_0(hit_3, Ray_intersect_2(_S29, _S41));
        var i_3 : i32 = i_0 + i32(1);
        hit_3 = _S42;
        i_0 = i_3;
    }
    return hit_3;
}

fn Spectrum_rgbToSpectralRadiance_0( _S43 : f32,  _S44 : vec3<f32>) -> f32
{
    var translated_0 : f32 = rcp_0(5.0f) * clamp(_S43 - 380.0f, 0.0f, 400.0f);
    var icoord_0 : i32 = i32(translated_0);
    var _S45 : f32 = fract(translated_0);
    var _S46 : vec4<f32> = (textureLoad((entryPointParams_spectrum_rgbToSpectralIntensityLut_0), (icoord_0)));
    var _S47 : vec3<f32> = _S46.xyz;
    var _S48 : vec4<f32> = (textureLoad((entryPointParams_spectrum_rgbToSpectralIntensityLut_0), (icoord_0 + i32(1))));
    return dot(_S44, mix(_S47, _S48.xyz, vec3<f32>(_S45)));
}

fn Material_evaluateBrdf_0( _S49 : Material_0,  _S50 : Hit_0,  _S51 : f32,  _S52 : ptr<function, Random_0>,  _S53 : ptr<function, Ray_0>) -> f32
{
    if((_S49.type_0) == u32(0))
    {
        var _S54 : f32 = Spectrum_rgbToSpectralRadiance_0(_S51, Material_getAlbedo_0(_S49));
        var _S55 : vec3<f32> = _S50.position_2 + _S50.normal_0 * vec3<f32>(0.00009999999747379f);
        var _S56 : vec3<f32> = Random_getCosineVector_0(&((*_S52)), _S50.normal_0);
        (*_S53) = Ray_x24init_0(_S55, _S56);
        return _S54;
    }
    else
    {
        var _S57 : vec3<f32> = vec3<f32>(0.0f);
        (*_S53) = Ray_x24init_0(_S57, _S57);
        return 0.0f;
    }
}

fn pathtrace_0( _S58 : Ray_0,  _S59 : f32,  _S60 : ptr<function, Random_0>) -> f32
{
    var _S61 : Ray_0 = _S58;
    var i_4 : i32 = i32(0);
    var throughput_0 : f32 = 1.0f;
    var radiance_0 : f32 = 0.0f;
    for(;;)
    {
        if(i_4 < i32(100))
        {
        }
        else
        {
            break;
        }
        var _S62 : Hit_0 = Objects_getHit_0(_S61);
        if(!_S62.success_0)
        {
            var _S63 : vec3<f32> = sky_0(_S61, &((*_S60)));
            radiance_0 = radiance_0 + throughput_0 * Spectrum_rgbToSpectralRadiance_0(_S59, _S63);
            break;
        }
        var _S64 : Material_0 = Material_0( entryPointParams_objects_materials_0[_S62.materialIndex_3].albedo_0, entryPointParams_objects_materials_0[_S62.materialIndex_3].roughness_0, entryPointParams_objects_materials_0[_S62.materialIndex_3].emission_0, entryPointParams_objects_materials_0[_S62.materialIndex_3].ior_0, entryPointParams_objects_materials_0[_S62.materialIndex_3].type_0 );
        var nextRay_0 : Ray_0;
        var _S65 : f32 = Material_evaluateBrdf_0(_S64, _S62, _S59, &((*_S60)), &(nextRay_0));
        var radiance_1 : f32 = radiance_0 + throughput_0 * Spectrum_rgbToSpectralRadiance_0(_S59, Material_getEmission_0(_S64));
        var throughput_1 : f32 = throughput_0 * _S65;
        var inverseTerminationProbability_0 : f32 = clamp(throughput_1, 0.0f, 1.0f);
        var _S66 : f32 = Random_getFloat_0(&((*_S60)));
        if(_S66 > inverseTerminationProbability_0)
        {
            radiance_0 = radiance_1;
            break;
        }
        var throughput_2 : f32 = throughput_1 * (1.0f / inverseTerminationProbability_0);
        var _S67 : Ray_0 = nextRay_0;
        var i_5 : i32 = i_4 + i32(1);
        _S61 = _S67;
        i_4 = i_5;
        throughput_0 = throughput_2;
        radiance_0 = radiance_1;
    }
    return radiance_0;
}

fn Spectrum_spectralRadianceToRgb_0( _S68 : f32,  _S69 : f32) -> vec3<f32>
{
    var translated_1 : f32 = rcp_0(1.0f) * clamp(_S68 - 360.0f, 0.0f, 471.0f);
    var icoord_1 : i32 = i32(translated_1);
    var _S70 : f32 = fract(translated_1);
    var _S71 : vec4<f32> = (textureLoad((entryPointParams_spectrum_wavelengthToXyzLut_0), (icoord_1)));
    var _S72 : vec3<f32> = _S71.xyz;
    var _S73 : vec4<f32> = (textureLoad((entryPointParams_spectrum_wavelengthToXyzLut_0), (icoord_1 + i32(1))));
    return xyzToRgb_0(mix(_S72, _S73.xyz, vec3<f32>(_S70))) * vec3<f32>(_S69);
}

@compute
@workgroup_size(8, 8, 1)
fn compute(@builtin(global_invocation_id) globalInvocationId_0 : vec3<u32>, @builtin(local_invocation_id) localInvocationId_0 : vec3<u32>, @builtin(local_invocation_index) localInvocationIndex_0 : u32)
{
    var _S74 : u32 = globalInvocationId_0.x;
    var _S75 : bool;
    if(_S74 >= (entryPointParams_screen_view_0.width_0))
    {
        _S75 = true;
    }
    else
    {
        _S75 = (globalInvocationId_0.y) >= (entryPointParams_screen_view_0.height_0);
    }
    if(_S75)
    {
        return;
    }
    var random_1 : Random_0 = Random_x24init_0(_S74 + globalInvocationId_0.y * entryPointParams_screen_view_0.width_0 + entryPointParams_screen_view_0.width_0 * entryPointParams_screen_view_0.height_0 * entryPointParams_screen_view_0.frameCount_0);
    var _S76 : vec2<u32> = globalInvocationId_0.xy;
    var _S77 : vec2<f32> = vec2<f32>(_S76) / vec2<f32>(f32(entryPointParams_screen_view_0.width_0), f32(entryPointParams_screen_view_0.height_0));
    var texcoord_0 : vec2<f32> = _S77;
    texcoord_0[i32(1)] = 1.0f - _S77.y;
    var _S78 : bool = Screen_shouldAccumulate_0();
    if(_S78)
    {
        texcoord_0 = texcoord_0 + getTaaOffset_0(entryPointParams_screen_view_0.frameCount_0) / vec2<f32>(f32(entryPointParams_screen_view_0.width_0), f32(entryPointParams_screen_view_0.height_0));
    }
    var ray_4 : Ray_0 = Ray_x24init_0(entryPointParams_screen_camera_0.position_0, Camera_viewDir_0(texcoord_0));
    var _S79 : f32 = Spectrum_generateWavelength_0(&(random_1));
    var _S80 : f32 = pathtrace_0(ray_4, _S79, &(random_1));
    var _S81 : vec3<f32> = Spectrum_spectralRadianceToRgb_0(_S79, _S80);
    var _S82 : vec3<i32> = vec3<i32>(vec2<i32>(_S76), i32(0));
    var previousSample_0 : vec4<f32> = (textureLoad((entryPointParams_textures_previous_0), ((_S82)).xy, ((_S82)).z));
    var previousColor_0 : vec3<f32> = previousSample_0.xyz;
    var frameAge_0 : f32 = previousSample_0.w;
    var color_0 : vec3<f32>;
    var frameAge_1 : f32;
    if(_S78)
    {
        var _S83 : f32 = frameAge_0 + 1.0f;
        color_0 = mix(previousColor_0, _S81, vec3<f32>((1.0f / _S83)));
        frameAge_1 = _S83;
    }
    else
    {
        color_0 = _S81;
        frameAge_1 = 0.0f;
    }
    textureStore((entryPointParams_textures_current_0), (_S76), (vec4<f32>(max(vec3<f32>(0.0f), color_0), frameAge_1)));
    return;
}

