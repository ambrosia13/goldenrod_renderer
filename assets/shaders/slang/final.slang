module copy;

import "lib/header";

struct Source {
    Texture2D texture;
    SamplerState sampler;
}

public struct CameraResponse {
    Texture1D<float> red;
    Texture1D<float> green;
    Texture1D<float> blue;

    SamplerState sampler;

    static const float ISO = 1.0;

    float normalizeIrradiance(float irradiance) {
        return clamp(irradiance * ISO, 0.0, 1.0);
    }

    float irradianceToIntensityRed(float irradiance) {
        return red.Sample(sampler, normalizeIrradiance(irradiance));
    }

    float irradianceToIntensityGreen(float irradiance) {
        return green.Sample(sampler, normalizeIrradiance(irradiance));
    }

    float irradianceToIntensityBlue(float irradiance) {
        return blue.Sample(sampler, normalizeIrradiance(irradiance));
    }

    public float3 toneMap(float3 irradiance) {
        return float3(
            irradianceToIntensityRed(irradiance.r),
            irradianceToIntensityGreen(irradiance.g),
            irradianceToIntensityBlue(irradiance.b)
        );
    }
}

struct Display {
    float2 effectiveStart;
    float2 effectiveEnd;
}

static const float3x3 FRX_ACES_INPUT_MATRIX = transpose(float3x3(
    float3(0.59719, 0.07600, 0.02840),
    float3(0.35458, 0.90834, 0.13383),
    float3(0.04823, 0.01566, 0.83777)
));

// ODT_SAT => XYZ => D60_2_D65 => sRGB
static const float3x3 FRX_ACES_OUTPUT_MATRIX = transpose(float3x3(
    float3(1.60475, -0.10208, -0.00327),
    float3(-0.53108, 1.10813, -0.07276),
    float3(-0.07367, -0.00605, 1.07602)
));

float3 FRX_RRT_AND_ODTF_FIT(float3 v) {
    float3 a = v * (v + 0.0245786) - 0.000090537;
    float3 b = v * (0.983729 * v + 0.4329510) + 0.238081;

    return a / b;
}

float3 frx_toneMap(float3 color) {
    color = mul(FRX_ACES_INPUT_MATRIX, color);
    color = FRX_RRT_AND_ODTF_FIT(color);

    return mul(FRX_ACES_OUTPUT_MATRIX, color);
}

[[shader("fragment")]]
float4 fragment(
    VertexOutput input,
    uniform ParameterBlock<Source> source,
    uniform ParameterBlock<CameraResponse> cameraResponse,
    uniform ParameterBlock<Display> display,
): SV_Target {
    float2 uv = input.uv;

    uv -= display.effectiveStart;
    uv /= (display.effectiveEnd) - (display.effectiveStart);

    float4 sample = source.texture.Sample(source.sampler, uv);
    sample.rgb = pow(sample.rgb, float3(1.0 / 2.2));
    //sample.rgb = frx_toneMap(sample.rgb);

    //sample.rgb = cameraResponse.toneMap(sample.rgb);

    return sample;
}