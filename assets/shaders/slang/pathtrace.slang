module pathtrace;

import lib.header;
import lib.intersect;
import lib.material;
import lib.object;
import lib.space;
import lib.noise;
import lib.spectrum;
import lib.atmosphere;

public struct MaterialTextures {
    public RWTexture2D<float4> current;
    public Texture2D<float4> previous;
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
            float atmosphereRadiance = 0.1;//pathtraceAtmosphere(ray, wavelength, random);
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

float3 stylizedAtmosphere(Ray ray) {
    float3 lightDir = float3(0.1, 0.2, -0.8);

    float planetRadius = 90.0;
    float atmosphereRadius = 100.0;
    float3 atmosphereOrigin = float3(0.0, -lerp(planetRadius, atmosphereRadius, 0.2), 0.0);
    
    Sphere planet = Sphere(atmosphereOrigin, planetRadius);
    Sphere atmosphere = Sphere(atmosphereOrigin, atmosphereRadius);

    Hit planetHit = ray.intersect(planet);
    Hit atmosphereHit = ray.intersect(atmosphere);

    float NdotH = clamp(ray.dir.y, 0.0, 1.0);
    float NdotL = clamp(dot(ray.dir, lightDir), 0.0, 1.0);

    float3 atmosphereColor = float3(0.2, 0.3, 1.0);
    float3 horizonColor = float3(0.7, 0.8, 0.9);
    float3 groundColor = float3(0.1);

    float3 color = float3(0.0);

    if (planetHit.success) {
        float fogDensity = 0.01;
        float fogFactor = exp(-planetHit.distance * fogDensity);

        color = lerp(horizonColor, groundColor, fogFactor);
    } else if (atmosphereHit.success) {
        float turbidity = 0.01;
        float opticalDepth = exp(-atmosphereHit.distance * turbidity);

        color = lerp(horizonColor, atmosphereColor, opticalDepth);
    }

    return color;
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

    uniform ParameterBlock<MaterialTextures> textures,

    uint3 globalInvocationId: SV_DispatchThreadID,
    uint3 localInvocationId: SV_GroupThreadID,
    uint localInvocationIndex: SV_GroupIndex,
) {
    if (globalInvocationId.x >= screen.view.rendererViewportWidth || globalInvocationId.y >= screen.view.rendererViewportHeight) {
        return;
    }

    uint pixelId = globalInvocationId.x + globalInvocationId.y * screen.view.rendererViewportWidth;
    uint frameOffset = screen.view.rendererViewportWidth * screen.view.rendererViewportHeight * screen.view.frameCount;
    Random random = Random(pixelId + frameOffset);

    float2 texcoord = float2(globalInvocationId.xy) / float2(screen.view.rendererViewportWidth, screen.view.rendererViewportHeight);
    texcoord.y = 1.0 - texcoord.y;

    if (screen.shouldAccumulate()) {
        // if the screen can accumulate, jitter the texcoord to avoid aliasing
        texcoord += getTaaOffset(screen.view.frameCount) / float2(screen.view.rendererViewportHeight, screen.view.rendererViewportHeight);
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
    accumulatedColor = stylizedAtmosphere(ray);

    textures.current.Store(globalInvocationId.xy, float4(accumulatedColor, frameAge));
}