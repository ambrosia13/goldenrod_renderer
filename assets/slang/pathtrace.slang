module pathtrace;

import lib.header;
import lib.intersect;
import lib.material;
import lib.object;
import lib.space;
import lib.noise;

struct Textures {
    RWTexture2D<float4> current;
    Texture2D<float4> previous;
}

float3 pathtrace(Ray ray, in Objects objects, inout Random random) {
    // ray energy and final color
    float3 throughput = float3(1.0);
    float3 radiance = float3(0.0);

    const int maxBounces = 5;

    for (int i = 0; i < maxBounces; ++i) {
        Hit hit = objects.getHit(ray);

        if (!hit.success) {
            // hit sky
            radiance += throughput * float3(0.0);
            break;
        }

        Material material = objects.materials[hit.materialIndex];

        Ray nextRay;
        float3 brdf = material.evaluateBrdf(hit, random, nextRay);

        radiance += throughput * material.getEmission();
        throughput *= brdf / PI;
        
        ray = nextRay;
    }

    return radiance;
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
    float3 sceneSpacePos = screen.camera.screenToScene(screenSpacePos);
    float3 viewDir = normalize(sceneSpacePos);

    Random random = Random(screen, globalInvocationId.xy);
    Ray ray = Ray(screen.camera.position, viewDir);

    float3 color = pathtrace(ray, objects, random);

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