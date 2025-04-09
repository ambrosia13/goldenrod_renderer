@binding(0) @group(0) var entryPointParams_source_texture_0 : texture_2d<f32>;

@binding(1) @group(0) var entryPointParams_source_sampler_0 : sampler;

@binding(0) @group(1) var entryPointParams_cameraResponse_red_0 : texture_1d<f32>;

@binding(1) @group(1) var entryPointParams_cameraResponse_green_0 : texture_1d<f32>;

@binding(2) @group(1) var entryPointParams_cameraResponse_blue_0 : texture_1d<f32>;

@binding(3) @group(1) var entryPointParams_cameraResponse_sampler_0 : sampler;

struct Display_std140_0
{
    @align(16) effectiveStart_0 : vec2<f32>,
    @align(8) effectiveEnd_0 : vec2<f32>,
};

@binding(0) @group(2) var<uniform> entryPointParams_display_0 : Display_std140_0;
fn CameraResponse_normalizeIrradiance_0( this_red_0 : texture_1d<f32>,  this_green_0 : texture_1d<f32>,  this_blue_0 : texture_1d<f32>,  this_sampler_0 : sampler,  irradiance_0 : f32) -> f32
{
    return clamp(irradiance_0, 0.0f, 1.0f);
}

fn CameraResponse_irradianceToIntensityRed_0( this_red_1 : texture_1d<f32>,  this_green_1 : texture_1d<f32>,  this_blue_1 : texture_1d<f32>,  this_sampler_1 : sampler,  irradiance_1 : f32) -> f32
{
    return (textureSample((this_red_1), (this_sampler_1), (CameraResponse_normalizeIrradiance_0(this_red_1, this_green_1, this_blue_1, this_sampler_1, irradiance_1))).x);
}

fn CameraResponse_irradianceToIntensityGreen_0( this_red_2 : texture_1d<f32>,  this_green_2 : texture_1d<f32>,  this_blue_2 : texture_1d<f32>,  this_sampler_2 : sampler,  irradiance_2 : f32) -> f32
{
    return (textureSample((this_green_2), (this_sampler_2), (CameraResponse_normalizeIrradiance_0(this_red_2, this_green_2, this_blue_2, this_sampler_2, irradiance_2))).x);
}

fn CameraResponse_irradianceToIntensityBlue_0( this_red_3 : texture_1d<f32>,  this_green_3 : texture_1d<f32>,  this_blue_3 : texture_1d<f32>,  this_sampler_3 : sampler,  irradiance_3 : f32) -> f32
{
    return (textureSample((this_blue_3), (this_sampler_3), (CameraResponse_normalizeIrradiance_0(this_red_3, this_green_3, this_blue_3, this_sampler_3, irradiance_3))).x);
}

fn CameraResponse_toneMap_0( this_red_4 : texture_1d<f32>,  this_green_4 : texture_1d<f32>,  this_blue_4 : texture_1d<f32>,  this_sampler_4 : sampler,  irradiance_4 : vec3<f32>) -> vec3<f32>
{
    return vec3<f32>(CameraResponse_irradianceToIntensityRed_0(this_red_4, this_green_4, this_blue_4, this_sampler_4, irradiance_4.x), CameraResponse_irradianceToIntensityGreen_0(this_red_4, this_green_4, this_blue_4, this_sampler_4, irradiance_4.y), CameraResponse_irradianceToIntensityBlue_0(this_red_4, this_green_4, this_blue_4, this_sampler_4, irradiance_4.z));
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
    var _S4 : vec3<f32> = CameraResponse_toneMap_0(entryPointParams_cameraResponse_red_0, entryPointParams_cameraResponse_green_0, entryPointParams_cameraResponse_blue_0, entryPointParams_cameraResponse_sampler_0, sample_0.xyz);
    sample_0.x = _S4.x;
    sample_0.y = _S4.y;
    sample_0.z = _S4.z;
    var _S5 : pixelOutput_0 = pixelOutput_0( sample_0 );
    return _S5;
}

