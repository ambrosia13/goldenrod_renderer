module pathtrace;

import "lib/header";
import "lib/intersect";
import "lib/material";
import "lib/object";
import "lib/space";

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

    float3 screenSpacePos = float3(texcoord, 1.0);
    float3 worldSpacePos = fromScreenSpace(screenSpacePos, screen.camera.inverseViewProjectionMatrix);
    float3 sceneSpacePos = worldSpacePos - screen.camera.position;

    float3 viewDir = normalize(sceneSpacePos);
    Ray ray = Ray(screen.camera.position, viewDir);

    Hit hit = objects.getHit(ray);
    float3 color = hit.success ? objects.materials[hit.materialIndex].albedo * max(0.0, dot(hit.normal, normalize(float3(0.2, 0.4, -0.7)))) : ray.dir;

    float4 previousSample = textures.previous.Load(int3(globalInvocationId.xy, 0));
    float3 previousColor = previousSample.rgb;
    float frameAge = previousSample.a;

    if (screen.shouldAccumulate()) {
        color = lerp(previousColor, color, 1.0 / (frameAge + 1.0));
        frameAge += 1.0;
    } else {
        // reset frame age so we don't accumulate next frame
        frameAge = 0.0;
    }

    textures.current.Store(globalInvocationId.xy, float4(color, frameAge));
}