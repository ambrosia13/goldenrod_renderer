@binding(0) @group(0) var entryPointParams_source_texture_0 : texture_2d<f32>;

@binding(1) @group(0) var entryPointParams_source_sampler_0 : sampler;

struct Display_std140_0
{
    @align(16) effectiveStart_0 : vec2<f32>,
    @align(8) effectiveEnd_0 : vec2<f32>,
};

@binding(0) @group(1) var<uniform> entryPointParams_display_0 : Display_std140_0;
fn FRX_RRT_AND_ODTF_FIT_0( v_0 : vec3<f32>) -> vec3<f32>
{
    return (v_0 * (v_0 + vec3<f32>(0.02457859925925732f)) - vec3<f32>(0.0000905370034161f)) / (v_0 * (vec3<f32>(0.98372900485992432f) * v_0 + vec3<f32>(0.43295100331306458f)) + vec3<f32>(0.23808099329471588f));
}

fn frx_toneMap_0( color_0 : vec3<f32>) -> vec3<f32>
{
    return (((mat3x3<f32>(vec3<f32>(1.60475003719329834f, -0.10208000242710114f, -0.00326999998651445f), vec3<f32>(-0.53108000755310059f, 1.10812997817993164f, -0.07276000082492828f), vec3<f32>(-0.07366999983787537f, -0.00604999996721745f, 1.0760200023651123f))) * (FRX_RRT_AND_ODTF_FIT_0((((mat3x3<f32>(vec3<f32>(0.59719002246856689f, 0.07599999755620956f, 0.0284000001847744f), vec3<f32>(0.35457998514175415f, 0.9083399772644043f, 0.1338299959897995f), vec3<f32>(0.04822999984025955f, 0.01565999910235405f, 0.83776998519897461f))) * (color_0)))))));
}

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
    var _S2 : vec4<f32> = (textureSample((entryPointParams_source_texture_0), (entryPointParams_source_sampler_0), ((_S1.uv_0 - entryPointParams_display_0.effectiveStart_0) / (entryPointParams_display_0.effectiveEnd_0 - entryPointParams_display_0.effectiveStart_0))));
    var sample_0 : vec4<f32> = _S2;
    var _S3 : vec3<f32> = pow(_S2.xyz, vec3<f32>(0.45454543828964233f));
    sample_0.x = _S3.x;
    sample_0.y = _S3.y;
    sample_0.z = _S3.z;
    var _S4 : vec3<f32> = frx_toneMap_0(sample_0.xyz);
    sample_0.x = _S4.x;
    sample_0.y = _S4.y;
    sample_0.z = _S4.z;
    var _S5 : pixelOutput_0 = pixelOutput_0( sample_0 );
    return _S5;
}

