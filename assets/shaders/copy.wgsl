@binding(0) @group(0) var entryPointParams_source_texture_0 : texture_2d<f32>;

@binding(1) @group(0) var entryPointParams_source_sampler_0 : sampler;

struct Display_std140_0
{
    @align(16) effectiveStart_0 : vec2<f32>,
    @align(8) effectiveEnd_0 : vec2<f32>,
};

@binding(0) @group(1) var<uniform> entryPointParams_display_0 : Display_std140_0;
struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_0 : vec2<f32>,
    @location(1) texcoord_0 : vec2<f32>,
};

@fragment
fn fragment( _S1 : pixelInput_0, @builtin(position) clipPosition_0 : vec4<f32>) -> pixelOutput_0
{
    var _S2 : pixelOutput_0 = pixelOutput_0( (textureSample((entryPointParams_source_texture_0), (entryPointParams_source_sampler_0), ((_S1.uv_0 - entryPointParams_display_0.effectiveStart_0) / (entryPointParams_display_0.effectiveEnd_0 - entryPointParams_display_0.effectiveStart_0)))) );
    return _S2;
}

