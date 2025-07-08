module copy;

import "lib/header";

public struct CameraResponse {
    Texture1D<float> red;
    Texture1D<float> green;
    Texture1D<float> blue;

    SamplerState sampler;

    static const float ISO = 1.0;

    float normalizeIrradiance(float luminance) {
        return clamp(luminance * ISO, 0.0, 1.0);
    }

    float irradianceToIntensityRed(float luminance) {
        return red.Sample(sampler, normalizeIrradiance(luminance));
    }

    float irradianceToIntensityGreen(float luminance) {
        return green.Sample(sampler, normalizeIrradiance(luminance));
    }

    float irradianceToIntensityBlue(float luminance) {
        return blue.Sample(sampler, normalizeIrradiance(luminance));
    }

    public float3 toneMap(float3 luminance) {
        return float3(
            irradianceToIntensityRed(luminance.r),
            irradianceToIntensityGreen(luminance.g),
            irradianceToIntensityBlue(luminance.b)
        );
    }
}

struct RendererViewport {
    uint2 start;
    uint2 end;
}

struct Display {
    Texture2D inputTexture;
    SamplerState inputSampler;
    ConstantBuffer<RendererViewport> viewport;
}

static const float3x3 FRX_ACES_INPUT_MATRIX = float3x3(
    float3(0.59719, 0.07600, 0.02840),
    float3(0.35458, 0.90834, 0.13383),
    float3(0.04823, 0.01566, 0.83777)
);

// ODT_SAT => XYZ => D60_2_D65 => sRGB
static const float3x3 FRX_ACES_OUTPUT_MATRIX = float3x3(
    float3(1.60475, -0.10208, -0.00327),
    float3(-0.53108, 1.10813, -0.07276),
    float3(-0.07367, -0.00605, 1.07602)
);

float3 FRX_RRT_AND_ODTF_FIT(float3 v) {
    float3 a = v * (v + 0.0245786) - 0.000090537;
    float3 b = v * (0.983729 * v + 0.4329510) + 0.238081;

    return a / b;
}

float3 frx_toneMap(float3 color) {
    color = mul(color, FRX_ACES_INPUT_MATRIX);
    color = FRX_RRT_AND_ODTF_FIT(color);

    return mul(color, FRX_ACES_OUTPUT_MATRIX);
}

[[shader("fragment")]]
float4 fragment(
    VertexOutput input,
    uniform ParameterBlock<Screen> screen,
    uniform ParameterBlock<CameraResponse> cameraResponse,
    uniform ParameterBlock<Display> display,
): SV_Target {
    float2 uv = input.uv;
    uint2 screenSize = uint2(screen.view.width, screen.view.height);

    uv -= float2(display.viewport.start) / float2(screenSize);
    uv /= (float2(display.viewport.end) / float2(screenSize)) - (float2(display.viewport.start) / float2(screenSize));

    if (any(clamp(uv, float2(0.0), float2(1.0)) != uv)) {
        return float4(0.0);
    }

    float4 sample = display.inputTexture.Sample(display.inputSampler, uv);
    sample = max(float4(0.0), sample);

    sample.rgb = pow(sample.rgb, float3(1.0 / 2.2));
    
    sample.rgb = frx_toneMap(sample.rgb);
    //sample.rgb = cameraResponse.toneMap(sample.rgb);

    return sample;
}