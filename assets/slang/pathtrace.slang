module pathtrace;

import "lib/header";
import "lib/intersect";
import "lib/material";
import "lib/object";

[[numthreads(8, 8, 1)]]
[[shader("compute")]]
void compute(
    uniform ParameterBlock<Screen> screen,
    uniform ParameterBlock<Objects> objects,

    uniform ParameterBlock<RWTexture2D<float4>> colorTexture,

    uint3 globalInvocationIndex: SV_DispatchThreadID,
    uint3 localInvocationIndex: SV_GroupThreadID,
) {
    if (globalInvocationIndex.x >= screen.view.width || globalInvocationIndex.y >= screen.view.height) {
        return;
    }

    float2 texcoord = float2(globalInvocationIndex.x, globalInvocationIndex.y) / float2(screen.view.width, screen.view.height);
    texcoord.y = 1.0 - texcoord.y;

    colorTexture.Store(globalInvocationIndex.xy, float4(texcoord.xy, 0.0, 1.0));
}