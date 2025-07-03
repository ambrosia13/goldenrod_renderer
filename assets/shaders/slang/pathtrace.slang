module pathtrace;

import lib.header;
import lib.intersect;
import lib.material;
import lib.object;
import lib.space;
import lib.noise;
import lib.spectrum;
import lib.atmosphere;

struct Textures {
    RWTexture2D<float4> current;
    Texture2D<float4> previous;
}

float3 sky(in Ray ray, inout Random random) {
    return float3(0.5);
}

float pathtrace(Ray ray, float wavelength, in Spectrum spectrum, in Objects objects, inout Random random) {
    // ray energy and final color
    float throughput = 1.0;
    float radiance = 0.0;

    const int maxBounces = 100;

    for (int i = 0; i < maxBounces; ++i) {
        Hit hit = objects.getHit(ray);

        if (!hit.success) {
            // hit sky
            float atmosphereRadiance = pathtraceAtmosphere(ray, wavelength, random);
            radiance += throughput * atmosphereRadiance;
            break;
        }

        Material material = objects.materials[hit.materialIndex];

        Ray nextRay;
        float brdf = material.evaluateBrdf(hit, wavelength, spectrum, random, nextRay);

        radiance += throughput * spectrum.rgbToSpectralRadiance(wavelength, material.getEmission());
        throughput *= brdf;

        // russian roulette path termination
        // probability is higher the lower the ray energy (throughput) is
        float inverseTerminationProbability = clamp(throughput, 0.0, 1.0);
        if (random.getFloat() > inverseTerminationProbability) {
            break;
        }

        // compensate for the energy lost with the terminated paths
        throughput *= 1.0 / inverseTerminationProbability;
        
        ray = nextRay;
    }

    return radiance;
}

float3 accumulate(float3 current, float3 previous, inout float frameAge, in Screen screen) {
    if (screen.view.frameCount > 0 && screen.shouldAccumulate()) {
        float3 accumulated = (previous * frameAge + current) / (frameAge + 1.0);
        frameAge += 1.0;

        return accumulated;
    } else {
        // reset frame age
        frameAge = 0.0;
        return current;
    }
}

[[numthreads(8, 8, 1)]]
[[shader("compute")]]
void compute(
    uniform ParameterBlock<Screen> screen,
    uniform ParameterBlock<Objects> objects,
    uniform ParameterBlock<Spectrum> spectrum,

    uniform ParameterBlock<Textures> textures,

    uint3 globalInvocationId: SV_DispatchThreadID,
    uint3 localInvocationId: SV_GroupThreadID,
    uint localInvocationIndex: SV_GroupIndex,
) {
    if (globalInvocationId.x >= screen.view.width || globalInvocationId.y >= screen.view.height) {
        return;
    }

    uint pixelId = globalInvocationId.x + globalInvocationId.y * screen.view.width;
    uint frameOffset = screen.view.width * screen.view.height * screen.view.frameCount;
    Random random = Random(pixelId + frameOffset);

    float2 texcoord = float2(globalInvocationId.xy) / float2(screen.view.width, screen.view.height);
    texcoord.y = 1.0 - texcoord.y;

    if (screen.shouldAccumulate()) {
        // if the screen can accumulate, jitter the texcoord to avoid aliasing
        texcoord += getTaaOffset(screen.view.frameCount) / float2(screen.view.width, screen.view.height);
    }

    float3 viewDir = screen.camera.viewDir(texcoord);

    Ray ray = Ray(screen.camera.position, viewDir);

    float wavelength = spectrum.generateWavelength(random);
    float radiance = pathtrace(ray, wavelength, spectrum, objects, random);

    float3 color = spectrum.spectralRadianceToRgb(wavelength, radiance);
    color /= 10000.0;

    float4 previousSample = textures.previous.Load(int3(globalInvocationId.xy, 0));
    float3 previousColor = previousSample.rgb;
    float frameAge = previousSample.a;

    float3 accumulatedColor = accumulate(color, previousColor, frameAge, screen);

    textures.current.Store(globalInvocationId.xy, float4(accumulatedColor, frameAge));
}