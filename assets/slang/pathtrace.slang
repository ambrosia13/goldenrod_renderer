module pathtrace;

import "lib/header";
import "lib/intersect";
import "lib/material";
import "lib/object";

struct Textures {
    RWTexture2D<float4> current;
    Texture2D<float4> previous;
}

[[numthreads(8, 8, 1)]]
[[shader("compute")]]
void compute(
    uniform ParameterBlock<Screen> screen,
    uniform ParameterBlock<Objects> objects,

    uniform ParameterBlock<Textures> textures,

    uint3 globalInvocationId: SV_DispatchThreadID,
    uint3 localInvocationId: SV_GroupThreadID,
) {
    if (globalInvocationId.x >= screen.view.width || globalInvocationId.y >= screen.view.height) {
        return;
    }

    float2 texcoord = float2(globalInvocationId.x, globalInvocationId.y) / float2(screen.view.width, screen.view.height);
    texcoord.y = 1.0 - texcoord.y;

    float4 previousColor = textures.previous.Load(int3(globalInvocationId.xy, 0));
    textures.current.Store(globalInvocationId.xy, float4(texcoord.xy, previousColor.z, 1.0));
}