module copy;

import "lib/header";

struct Source {
    Texture2D texture;
    SamplerState sampler;
}

struct Display {
    float2 effectiveStart;
    float2 effectiveEnd;
}

[[shader("fragment")]]
float4 fragment(
    VertexOutput input,
    uniform ParameterBlock<Source> source,
    uniform ParameterBlock<Display> display,
): SV_Target {
    float2 uv = input.uv;

    uv -= display.effectiveStart;
    uv /= (display.effectiveEnd) - (display.effectiveStart);

    return source.texture.Sample(source.sampler, uv);
}