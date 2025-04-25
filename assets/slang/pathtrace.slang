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

float density(in float3 pos) {
    return all(abs(pos.xz) < float2(3.0)) ? 0.1 : 0.0;
}

float raytraceVolume(in Ray ray) {
    float b = -dot(ray.pos, ray.dir);
    float d = b * b - dot(ray.pos, ray.pos) + 1.3;

    if (d < 0.0) return -1.0;

    float s = sqrt(d);
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
            radiance += throughput * spectrum.rgbToSpectralRadiance(wavelength, sky(ray, random));
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

[[numthreads(8, 8, 1)]]
[[shader("compute")]]
void compute(
    uniform ParameterBlock<Screen> screen,
    uniform ParameterBlock<Objects> objects,

    uniform ParameterBlock<Textures> textures,
    uniform ParameterBlock<Spectrum> spectrum,

    uint3 globalInvocationId: SV_DispatchThreadID,
    uint3 localInvocationId: SV_GroupThreadID,
    uint localInvocationIndex: SV_GroupIndex,
) {
    if (globalInvocationId.x >= screen.view.width || globalInvocationId.y >= screen.view.height) {
        return;
    }

    float2 texcoord = float2(globalInvocationId.x, globalInvocationId.y) / float2(screen.view.width, screen.view.height);
    texcoord.y = 1.0 - texcoord.y;

    if (screen.shouldAccumulate()) {
        // if the screen can accumulate, jitter the texcoord to avoid aliasing
        texcoord += getTaaOffset(screen.view.frameCount) / float2(screen.view.width, screen.view.height);
    }

    float3 viewDir = screen.camera.viewDir(texcoord);

    Random random = Random(screen, globalInvocationId.xy);
    Ray ray = Ray(screen.camera.position, viewDir);

    float wavelength = spectrum.generateWavelength(random);
    float radiance = pathtrace(ray, wavelength, spectrum, objects, random);

    //float3 color = spectrum.spectralRadianceToRgb(wavelength, radiance);
    float3 color = spectrum.spectralRadianceToRgb(wavelength, transmittance(ray, wavelength, random));

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

    // make sure there's no negative color
    color = max(float3(0.0), color);

    textures.current.Store(globalInvocationId.xy, float4(color, frameAge));
}