struct View_std140_0
{
    @align(16) width_0 : u32,
    @align(4) height_0 : u32,
    @align(8) frameCount_0 : u32,
};

@binding(1) @group(0) var<uniform> entryPointParams_screen_view_0 : View_std140_0;
@binding(0) @group(2) var entryPointParams_textures_current_0 : texture_storage_2d<rgba32float, read_write>;

@binding(1) @group(2) var entryPointParams_textures_previous_0 : texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn compute(@builtin(global_invocation_id) globalInvocationId_0 : vec3<u32>, @builtin(local_invocation_id) localInvocationId_0 : vec3<u32>)
{
    var _S1 : u32 = globalInvocationId_0.x;
    var _S2 : bool;
    if(_S1 >= (entryPointParams_screen_view_0.width_0))
    {
        _S2 = true;
    }
    else
    {
        _S2 = (globalInvocationId_0.y) >= (entryPointParams_screen_view_0.height_0);
    }
    if(_S2)
    {
        return;
    }
    var _S3 : vec2<f32> = vec2<f32>(f32(_S1), f32(globalInvocationId_0.y)) / vec2<f32>(f32(entryPointParams_screen_view_0.width_0), f32(entryPointParams_screen_view_0.height_0));
    var texcoord_0 : vec2<f32> = _S3;
    texcoord_0[i32(1)] = 1.0f - _S3.y;
    var _S4 : vec2<u32> = globalInvocationId_0.xy;
    var _S5 : vec3<i32> = vec3<i32>(vec2<i32>(_S4), i32(0));
    textureStore((entryPointParams_textures_current_0), (_S4), (vec4<f32>(texcoord_0.xy, (textureLoad((entryPointParams_textures_previous_0), ((_S5)).xy, ((_S5)).z)).z, 1.0f)));
    return;
}

